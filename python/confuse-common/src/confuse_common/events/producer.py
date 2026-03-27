"""
Kafka Event Producer

A generic event producer that works with any event type.
"""

import logging
from typing import Optional, Any
from pydantic import BaseModel
from confluent_kafka import Producer

from .config import KafkaConfig, ConfigError

logger = logging.getLogger(__name__)


class ProducerError(Exception):
    """Producer error"""
    pass


class EventProducer:
    """Event producer for publishing events to Kafka"""
    
    def __init__(self, config: Optional[KafkaConfig] = None):
        """
        Create a new event producer
        
        Args:
            config: Kafka configuration. If None, will be loaded from environment.
        """
        self.config = config or KafkaConfig.from_env()
        self._producer: Optional[Producer] = None
    
    @classmethod
    def from_env(cls) -> "EventProducer":
        """Create a new event producer from environment configuration"""
        return cls(KafkaConfig.from_env())
    
    @property
    def producer(self) -> Producer:
        """Get or create the underlying Kafka producer"""
        if self._producer is None:
            producer_config = self.config.to_producer_config()
            self._producer = Producer(producer_config)
            logger.info(
                f"Created Kafka producer for {self.config.bootstrap_servers} "
                f"({self.config.client_id})"
            )
        return self._producer
    
    def publish(self, event: BaseModel) -> None:
        """
        Publish an event to its designated topic
        
        The event must be a Pydantic model with a static topic() method.
        """
        topic = event.topic() if hasattr(event, 'topic') else 'unknown'
        self.publish_to_topic(event, topic)
    
    def publish_to_topic(
        self,
        event: BaseModel,
        topic: str,
        key: Optional[str] = None,
    ) -> None:
        """
        Publish an event to a specific topic
        
        Args:
            event: Pydantic model to publish
            topic: Kafka topic name
            key: Optional message key
        """
        payload = event.model_dump_json()
        
        logger.debug(f"Publishing event to topic '{topic}': {len(payload)} bytes")
        
        self.producer.produce(
            topic=topic,
            value=payload.encode('utf-8'),
            key=key.encode('utf-8') if key else None,
            callback=self._delivery_callback,
        )
        
        # Trigger delivery reports
        self.producer.poll(0)
    
    def publish_raw(
        self,
        topic: str,
        value: str,
        key: Optional[str] = None,
    ) -> None:
        """
        Publish a raw string value to a topic
        
        Args:
            topic: Kafka topic name
            value: Raw string value
            key: Optional message key
        """
        self.producer.produce(
            topic=topic,
            value=value.encode('utf-8'),
            key=key.encode('utf-8') if key else None,
            callback=self._delivery_callback,
        )
        self.producer.poll(0)
    
    def flush(self, timeout: float = 30.0) -> int:
        """
        Wait for all buffered messages to be delivered
        
        Args:
            timeout: Maximum time to wait in seconds
            
        Returns:
            Number of messages still in queue (0 if all delivered)
        """
        return self.producer.flush(timeout)
    
    def _delivery_callback(self, err: Any, msg: Any) -> None:
        """Callback for delivery reports"""
        if err:
            logger.error(f"Message delivery failed: {err}")
        else:
            logger.debug(
                f"Message delivered to {msg.topic()} "
                f"[{msg.partition()}] @ {msg.offset()}"
            )
    
    def __enter__(self) -> "EventProducer":
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self.flush()
