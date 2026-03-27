/**
 * Kafka Event Producer
 *
 * A generic event producer that works with any event type.
 */

import { Kafka, Producer, ProducerRecord } from 'kafkajs';
import { KafkaConfig } from './config';

export class EventProducer {
    private config: KafkaConfig;
    private kafka: Kafka | null = null;
    private producer: Producer | null = null;

    constructor(config?: KafkaConfig) {
        this.config = config || KafkaConfig.fromEnv();
    }

    static fromEnv(): EventProducer {
        return new EventProducer(KafkaConfig.fromEnv());
    }

    private getProducer(): Producer {
        if (!this.producer) {
            const producerConfig = this.config.toProducerConfig() as any;
            this.kafka = new Kafka(producerConfig);
            this.producer = this.kafka.producer();
        }
        return this.producer;
    }

    /**
     * Connect to Kafka
     */
    async connect(): Promise<void> {
        await this.getProducer().connect();
        console.log(
            `Connected to Kafka at ${this.config.bootstrapServers} ` +
            `(${this.config.clientId})`
        );
    }

    /**
     * Disconnect from Kafka
     */
    async disconnect(): Promise<void> {
        if (this.producer) {
            await this.producer.disconnect();
            this.producer = null;
            this.kafka = null;
        }
    }

    /**
     * Publish an event to a topic
     */
    async publish<T extends object>(event: T, topic: string): Promise<void> {
        const payload = JSON.stringify(event);

        const record: ProducerRecord = {
            topic,
            messages: [{ value: payload }],
        };

        await this.getProducer().send(record);
        console.log(`Event published to topic '${topic}': ${payload.length} bytes`);
    }

    /**
     * Publish an event with a custom key
     */
    async publishWithKey<T extends object>(
        event: T,
        topic: string,
        key: string
    ): Promise<void> {
        const payload = JSON.stringify(event);

        const record: ProducerRecord = {
            topic,
            messages: [{ key, value: payload }],
        };

        await this.getProducer().send(record);
    }

    /**
     * Publish multiple events in a batch
     */
    async publishBatch<T extends object>(
        events: Array<{ event: T; topic: string; key?: string }>
    ): Promise<void> {
        const groupedByTopic = new Map<
            string,
            Array<{ key?: string; value: string }>
        >();

        for (const { event, topic, key } of events) {
            const payload = JSON.stringify(event);
            if (!groupedByTopic.has(topic)) {
                groupedByTopic.set(topic, []);
            }
            groupedByTopic.get(topic)!.push({ key, value: payload });
        }

        const records = Array.from(groupedByTopic.entries()).map(
            ([topic, messages]) => ({
                topic,
                messages,
            })
        );

        await this.getProducer().sendBatch({ topicMessages: records });
    }
}
