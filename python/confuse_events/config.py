"""
Kafka Configuration with CONFLUENT_* Environment Variable Support

This module provides a unified configuration builder that works with both
local Kafka (development) and Confluent Cloud (production).
"""

import os
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
import logging

logger = logging.getLogger(__name__)


class Environment(Enum):
    """Environment mode for the service"""
    DEVELOPMENT = "development"
    PRODUCTION = "production"
    
    @classmethod
    def from_env(cls) -> "Environment":
        """Parse from ENVIRONMENT env var"""
        env_value = os.getenv("ENVIRONMENT", "development").lower()
        if env_value in ("production", "prod"):
            return cls.PRODUCTION
        return cls.DEVELOPMENT


class ConfigError(Exception):
    """Configuration error"""
    pass


@dataclass
class KafkaConfig:
    """
    Kafka configuration builder
    
    Uses CONFLUENT_* environment variables for configuration:
    - CONFLUENT_BOOTSTRAP_SERVERS: Kafka bootstrap servers
    - CONFLUENT_API_KEY: SASL username (Confluent Cloud API key)
    - CONFLUENT_API_SECRET: SASL password (Confluent Cloud API secret)
    - KAFKA_CLIENT_ID: Client ID for this service
    - KAFKA_GROUP_ID: Consumer group ID (for consumers)
    """
    
    bootstrap_servers: str
    security_protocol: str
    sasl_mechanism: Optional[str] = None
    sasl_username: Optional[str] = None
    sasl_password: Optional[str] = None
    client_id: str = "confuse-service"
    group_id: Optional[str] = None
    environment: Environment = field(default_factory=Environment.from_env)
    
    @classmethod
    def from_env(cls) -> "KafkaConfig":
        """
        Create a new KafkaConfig from environment variables
        
        In production mode, this will fail-fast if required CONFLUENT_* vars are missing.
        In development mode, it will use sensible defaults for local Kafka.
        """
        environment = Environment.from_env()
        
        # Get bootstrap servers (required)
        bootstrap_servers = os.getenv("CONFLUENT_BOOTSTRAP_SERVERS") or os.getenv("KAFKA_BOOTSTRAP_SERVERS")
        
        if not bootstrap_servers:
            if environment == Environment.PRODUCTION:
                raise ConfigError("Missing required environment variable: CONFLUENT_BOOTSTRAP_SERVERS")
            bootstrap_servers = "localhost:9092"
        
        # Get SASL credentials (required in production)
        sasl_username = os.getenv("CONFLUENT_API_KEY")
        sasl_password = os.getenv("CONFLUENT_API_SECRET")
        
        if environment == Environment.PRODUCTION:
            if not sasl_username:
                raise ConfigError("Missing required environment variable: CONFLUENT_API_KEY")
            if not sasl_password:
                raise ConfigError("Missing required environment variable: CONFLUENT_API_SECRET")
        
        # Determine security settings based on environment
        if environment == Environment.PRODUCTION or sasl_username:
            security_protocol = "SASL_SSL"
            sasl_mechanism = "PLAIN"
        else:
            security_protocol = "PLAINTEXT"
            sasl_mechanism = None
        
        # Get client and group IDs
        client_id = os.getenv("KAFKA_CLIENT_ID", "confuse-service")
        group_id = os.getenv("KAFKA_GROUP_ID")
        
        config = cls(
            bootstrap_servers=bootstrap_servers,
            security_protocol=security_protocol,
            sasl_mechanism=sasl_mechanism,
            sasl_username=sasl_username,
            sasl_password=sasl_password,
            client_id=client_id,
            group_id=group_id,
            environment=environment,
        )
        
        # Log configuration (without secrets)
        logger.info(
            f"Kafka config: bootstrap_servers={config.bootstrap_servers}, "
            f"security={config.security_protocol}, client_id={config.client_id}, "
            f"env={config.environment.value}"
        )
        
        return config
    
    def to_producer_config(self) -> dict:
        """Build a confluent-kafka producer configuration dict"""
        config = {
            "bootstrap.servers": self.bootstrap_servers,
            "client.id": self.client_id,
            "security.protocol": self.security_protocol,
            "acks": "all",
            "retries": 5,
            "retry.backoff.ms": 100,
            "request.timeout.ms": 30000,
            "enable.idempotence": True,
        }
        
        if self.sasl_mechanism:
            config["sasl.mechanism"] = self.sasl_mechanism
        
        if self.sasl_username:
            config["sasl.username"] = self.sasl_username
        
        if self.sasl_password:
            config["sasl.password"] = self.sasl_password
        
        return config
    
    def to_consumer_config(self) -> dict:
        """Build a confluent-kafka consumer configuration dict"""
        if not self.group_id:
            raise ConfigError("KAFKA_GROUP_ID is required for consumers")
        
        config = {
            "bootstrap.servers": self.bootstrap_servers,
            "client.id": self.client_id,
            "group.id": self.group_id,
            "security.protocol": self.security_protocol,
            "enable.auto.commit": False,
            "auto.offset.reset": "earliest",
            "session.timeout.ms": 45000,
        }
        
        if self.sasl_mechanism:
            config["sasl.mechanism"] = self.sasl_mechanism
        
        if self.sasl_username:
            config["sasl.username"] = self.sasl_username
        
        if self.sasl_password:
            config["sasl.password"] = self.sasl_password
        
        return config
    
    def validate(self) -> None:
        """Validate the configuration"""
        if not self.bootstrap_servers:
            raise ConfigError("bootstrap_servers cannot be empty")
        
        if self.environment == Environment.PRODUCTION:
            if not self.sasl_username or not self.sasl_password:
                raise ConfigError("SASL credentials are required in production")
