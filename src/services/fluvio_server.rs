use std::sync::Arc;
use fluvio::Fluvio;
use tonic::{Request, Response, Status};
use crate::proto::fluvio_server::fluvio_service_server::FluvioService;
use crate::proto::fluvio_server::*;
use crate::handlers::*;

pub struct FluvioServerService {
    #[allow(dead_code)]
    fluvio: Arc<Fluvio>,
}
impl FluvioServerService {
    pub fn new(fluvio: Arc<Fluvio>) -> Self {
        Self { fluvio }
    }
}

#[tonic::async_trait]
impl FluvioService for FluvioServerService {
    async fn produce(&self, request: Request<ProduceRequest>) -> Result<Response<ProduceReply>, Status> {
        produce(self, request).await
    }

    async fn batch_produce(&self, request: Request<BatchProduceRequest>) -> Result<Response<BatchProduceReply>, Status> {
        batch_produce(self, request).await
    }

    async fn consume(&self, request: Request<ConsumeRequest>) -> Result<Response<ConsumeReply>, Status> {
        consume(self, request).await
    }

    type StreamConsumeStream = StreamConsumeStream;

    async fn stream_consume(&self, request: Request<StreamConsumeRequest>) -> Result<Response<Self::StreamConsumeStream>, Status> {
        stream_consume(self, request).await
    }

    async fn commit_offset(&self, request: Request<CommitOffsetRequest>) -> Result<Response<CommitOffsetReply>, Status> {
        commit_offset(self, request).await
    }

    async fn create_topic(&self, request: Request<CreateTopicRequest>) -> Result<Response<CreateTopicReply>, Status> {
        create_topic(self, request).await
    }

    async fn delete_topic(&self, request: Request<DeleteTopicRequest>) -> Result<Response<DeleteTopicReply>, Status> {
        delete_topic(self, request).await
    }

    async fn list_topics(&self, request: Request<ListTopicsRequest>) -> Result<Response<ListTopicsReply>, Status> {
        list_topics(self, request).await
    }

    async fn describe_topic(&self, request: Request<DescribeTopicRequest>) -> Result<Response<DescribeTopicReply>, Status> {
        describe_topic(self, request).await
    }

    async fn list_consumer_groups(&self, request: Request<ListConsumerGroupsRequest>) -> Result<Response<ListConsumerGroupsReply>, Status> {
        list_consumer_groups(self, request).await
    }

    async fn describe_consumer_group(&self, request: Request<DescribeConsumerGroupRequest>) -> Result<Response<DescribeConsumerGroupReply>, Status> {
        describe_consumer_group(self, request).await
    }

    async fn create_smart_module(&self, request: Request<CreateSmartModuleRequest>) -> Result<Response<CreateSmartModuleReply>, Status> {
        create_smart_module(self, request).await
    }

    async fn delete_smart_module(&self, request: Request<DeleteSmartModuleRequest>) -> Result<Response<DeleteSmartModuleReply>, Status> {
        delete_smart_module(self, request).await
    }

    async fn list_smart_modules(&self, request: Request<ListSmartModulesRequest>) -> Result<Response<ListSmartModulesReply>, Status> {
        list_smart_modules(self, request).await
    }

    async fn describe_smart_module(&self, request: Request<DescribeSmartModuleRequest>) -> Result<Response<DescribeSmartModuleReply>, Status> {
        describe_smart_module(self, request).await
    }

    async fn update_smart_module(&self, request: Request<UpdateSmartModuleRequest>) -> Result<Response<UpdateSmartModuleReply>, Status> {
        update_smart_module(self, request).await
    }

    async fn health_check(&self, request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckReply>, Status> {
        health_check(self, request).await
    }
}