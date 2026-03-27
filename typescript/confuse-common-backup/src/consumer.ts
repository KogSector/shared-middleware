/**
 * Kafka Event Consumer
 *
 * A generic event consumer with handler interface for processing events.
 */

import { Kafka, Consumer, EachMessagePayload } from 'kafkajs';
import { KafkaConfig } from './config';

/**
 * Interface for event handlers
 */
export interface EventHandler {
    /**
     * Handle a message from Kafka
     */
    handle(topic: string, payload: Buffer): Promise<void>;

    /**
     * Handle errors during message processing
     */
    handleError(
        topic: string,
        error: Error,
        payload?: Buffer
    ): Promise<void>;
}

export class EventConsumer {
    private config: KafkaConfig;
    private kafka: Kafka | null = null;
    private consumer: Consumer | null = null;
    private isShuttingDown = false;

    constructor(config?: KafkaConfig) {
        this.config = config || KafkaConfig.fromEnv();
    }

    static fromEnv(): EventConsumer {
        return new EventConsumer(KafkaConfig.fromEnv());
    }

    private getConsumer(): Consumer {
        if (!this.consumer) {
            const consumerConfig = this.config.toConsumerConfig() as any;
            this.kafka = new Kafka({
                clientId: consumerConfig.clientId,
                brokers: consumerConfig.brokers,
                ssl: consumerConfig.ssl,
                sasl: consumerConfig.sasl,
            });
            this.consumer = this.kafka.consumer({
                groupId: consumerConfig.groupId,
            });
        }
        return this.consumer;
    }

    /**
     * Connect to Kafka
     */
    async connect(): Promise<void> {
        await this.getConsumer().connect();
        console.log(
            `Connected to Kafka at ${this.config.bootstrapServers} ` +
            `(group: ${this.config.groupId})`
        );
    }

    /**
     * Subscribe to topics
     */
    async subscribe(topics: string[]): Promise<void> {
        for (const topic of topics) {
            await this.getConsumer().subscribe({ topic, fromBeginning: false });
        }
        console.log(`Subscribed to topics: ${topics.join(', ')}`);
    }

    /**
     * Start consuming messages with the provided handler
     */
    async run(handler: EventHandler): Promise<void> {
        console.log('Starting consumer loop');

        await this.getConsumer().run({
            eachMessage: async (payload: EachMessagePayload) => {
                if (this.isShuttingDown) {
                    return;
                }

                const { topic, message } = payload;
                const value = message.value;

                if (!value) {
                    console.warn('Received message with no value');
                    return;
                }

                try {
                    await handler.handle(topic, value);
                } catch (error) {
                    console.error(`Error processing message from ${topic}:`, error);
                    await handler.handleError(topic, error as Error, value);
                }
            },
        });
    }

    /**
     * Signal the consumer to shut down
     */
    shutdown(): void {
        console.log('Shutdown requested');
        this.isShuttingDown = true;
    }

    /**
     * Disconnect from Kafka
     */
    async disconnect(): Promise<void> {
        if (this.consumer) {
            await this.consumer.disconnect();
            this.consumer = null;
            this.kafka = null;
            console.log('Consumer disconnected');
        }
    }
}

/**
 * Helper function to deserialize a message payload
 */
export function deserializeEvent<T>(payload: Buffer): T {
    return JSON.parse(payload.toString('utf-8')) as T;
}
