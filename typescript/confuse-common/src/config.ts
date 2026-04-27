/**
 * Kafka Configuration with CONFLUENT_* Environment Variable Support
 *
 * This module provides a unified configuration builder that works with both
 * local Kafka (development) and Confluent Cloud (production).
 */

export enum Environment {
    DEVELOPMENT = 'development',
    PRODUCTION = 'production',
}

export class ConfigError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'ConfigError';
    }
}

/**
 * Kafka configuration builder
 *
 * Uses CONFLUENT_* environment variables for configuration:
 * - CONFLUENT_BOOTSTRAP_SERVERS: Kafka bootstrap servers
 * - CONFLUENT_API_KEY: SASL username (Confluent Cloud API key)
 * - CONFLUENT_API_SECRET: SASL password (Confluent Cloud API secret)
 * - KAFKA_CLIENT_ID: Client ID for this service
 * - KAFKA_GROUP_ID: Consumer group ID (for consumers)
 */
export interface KafkaConfigOptions {
    bootstrapServers: string;
    securityProtocol: string;
    saslMechanism?: string;
    saslUsername?: string;
    saslPassword?: string;
    clientId: string;
    groupId?: string;
    environment: Environment;
}

export class KafkaConfig {
    public readonly bootstrapServers: string;
    public readonly securityProtocol: string;
    public readonly saslMechanism?: string;
    public readonly saslUsername?: string;
    public readonly saslPassword?: string;
    public readonly clientId: string;
    public readonly groupId?: string;
    public readonly environment: Environment;

    constructor(options: KafkaConfigOptions) {
        this.bootstrapServers = options.bootstrapServers;
        this.securityProtocol = options.securityProtocol;
        this.saslMechanism = options.saslMechanism;
        this.saslUsername = options.saslUsername;
        this.saslPassword = options.saslPassword;
        this.clientId = options.clientId;
        this.groupId = options.groupId;
        this.environment = options.environment;
    }

    /**
     * Create a new KafkaConfig from environment variables
     *
     * In production mode, this will fail-fast if required CONFLUENT_* vars are missing.
     * In development mode, it will use sensible defaults for local Kafka.
     */
    static fromEnv(): KafkaConfig {
        const environment = this.getEnvironment();

        // Get bootstrap servers (required)
        let bootstrapServers =
            process.env.CONFLUENT_BOOTSTRAP_SERVERS ||
            process.env.KAFKA_BOOTSTRAP_SERVERS ||
            '';

        if (!bootstrapServers) {
            if (environment === Environment.PRODUCTION) {
                throw new ConfigError(
                    'Missing required environment variable: CONFLUENT_BOOTSTRAP_SERVERS'
                );
            }
            bootstrapServers = '127.0.0.1:9092';
        }

        // Get SASL credentials (required in production)
        const saslUsername = process.env.CONFLUENT_API_KEY;
        const saslPassword = process.env.CONFLUENT_API_SECRET;

        if (environment === Environment.PRODUCTION) {
            if (!saslUsername) {
                throw new ConfigError(
                    'Missing required environment variable: CONFLUENT_API_KEY'
                );
            }
            if (!saslPassword) {
                throw new ConfigError(
                    'Missing required environment variable: CONFLUENT_API_SECRET'
                );
            }
        }

        // Determine security settings based on environment
        let securityProtocol: string;
        let saslMechanism: string | undefined;

        if (environment === Environment.PRODUCTION || saslUsername) {
            securityProtocol = 'SASL_SSL';
            saslMechanism = 'PLAIN';
        } else {
            securityProtocol = 'PLAINTEXT';
            saslMechanism = undefined;
        }

        // Get client and group IDs
        const clientId = process.env.KAFKA_CLIENT_ID || 'confuse-service';
        const groupId = process.env.KAFKA_GROUP_ID;

        const config = new KafkaConfig({
            bootstrapServers,
            securityProtocol,
            saslMechanism,
            saslUsername,
            saslPassword,
            clientId,
            groupId,
            environment,
        });

        console.log(
            `Kafka config: bootstrap_servers=${config.bootstrapServers}, ` +
            `security=${config.securityProtocol}, client_id=${config.clientId}, ` +
            `env=${config.environment}`
        );

        return config;
    }

    private static getEnvironment(): Environment {
        const envValue = (process.env.ENVIRONMENT || 'development').toLowerCase();
        if (envValue === 'production' || envValue === 'prod') {
            return Environment.PRODUCTION;
        }
        return Environment.DEVELOPMENT;
    }

    /**
     * Build a KafkaJS producer configuration
     */
    toProducerConfig(): object {
        const config: any = {
            clientId: this.clientId,
            brokers: this.bootstrapServers.split(','),
        };

        if (this.saslUsername && this.saslPassword) {
            config.ssl = true;
            config.sasl = {
                mechanism: 'plain',
                username: this.saslUsername,
                password: this.saslPassword,
            };
        }

        return config;
    }

    /**
     * Build a KafkaJS consumer configuration
     */
    toConsumerConfig(): object {
        if (!this.groupId) {
            throw new ConfigError('KAFKA_GROUP_ID is required for consumers');
        }

        const config: any = {
            clientId: this.clientId,
            brokers: this.bootstrapServers.split(','),
            groupId: this.groupId,
        };

        if (this.saslUsername && this.saslPassword) {
            config.ssl = true;
            config.sasl = {
                mechanism: 'plain',
                username: this.saslUsername,
                password: this.saslPassword,
            };
        }

        return config;
    }

    /**
     * Validate the configuration
     */
    validate(): void {
        if (!this.bootstrapServers) {
            throw new ConfigError('bootstrapServers cannot be empty');
        }

        if (this.environment === Environment.PRODUCTION) {
            if (!this.saslUsername || !this.saslPassword) {
                throw new ConfigError('SASL credentials are required in production');
            }
        }
    }
}
