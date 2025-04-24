use tonic::{Request, Response, Status};
use crate::proto::fluvio_server::*;
use crate::services::fluvio_server::FluvioServerService;
use futures_core::Stream;
use std::pin::Pin;
use fluvio::{Fluvio, FluvioAdmin};
use fluvio::metadata::topic::TopicSpec;
use fluvio::RecordKey;
use fluvio::Offset;
use tokio_stream::StreamExt;

pub type StreamConsumeStream = Pin<Box<dyn Stream<Item = Result<ConsumedMessage, Status>> + Send + 'static>>;

pub async fn produce(
    _service: &FluvioServerService,
    request: Request<ProduceRequest>,
) -> Result<Response<ProduceReply>, Status> {
    let req = request.into_inner();
    let fluvio = Fluvio::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let producer = fluvio.topic_producer(&req.topic).await.map_err(|e| Status::internal(e.to_string()))?;
    let record = req.message.as_bytes();
    let result = producer.send(RecordKey::NULL, record).await;
    match result {
        Ok(_) => Ok(Response::new(ProduceReply { success: true, error: "".to_string(), message_id: req.message_id })),
        Err(e) => Ok(Response::new(ProduceReply { success: false, error: e.to_string(), message_id: req.message_id })),
    }
}

pub async fn batch_produce(
    _service: &FluvioServerService,
    request: Request<BatchProduceRequest>,
) -> Result<Response<BatchProduceReply>, Status> {
    let req = request.into_inner();
    let fluvio = Fluvio::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let producer = fluvio.topic_producer(&req.topic).await.map_err(|e| Status::internal(e.to_string()))?;
    let mut success = Vec::new();
    let mut error = Vec::new();
    for msg in req.messages {
        let record = msg.message.as_bytes();
        match producer.send(RecordKey::NULL, record).await {
            Ok(_) => {
                success.push(true);
                error.push(String::new());
            },
            Err(e) => {
                success.push(false);
                error.push(e.to_string());
            }
        }
    }
    Ok(Response::new(BatchProduceReply { success, error }))
}

pub async fn consume(
    _service: &FluvioServerService,
    request: Request<ConsumeRequest>,
) -> Result<Response<ConsumeReply>, Status> {
    let req = request.into_inner();
    let fluvio = Fluvio::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let consumer = fluvio.partition_consumer(&req.topic, req.partition as u32).await.map_err(|e| Status::internal(e.to_string()))?;
    let mut stream = consumer.stream(Offset::absolute(req.offset).unwrap()).await.map_err(|e| Status::internal(e.to_string()))?;
    let mut messages = Vec::new();
    let mut next_offset = req.offset as i64;
    while let Some(Ok(record)) = stream.next().await {
        messages.push(ConsumedMessage {
            message: String::from_utf8_lossy(record.value()).to_string(),
            offset: record.offset(),
            key: String::new(),
            headers: Default::default(),
            timestamp: 0,
            message_id: String::new(),
            partition: req.partition,
        });
        next_offset = record.offset() as i64 + 1;
        if messages.len() as i32 >= req.max_messages { break; }
    }
    Ok(Response::new(ConsumeReply { messages, error: String::new(), next_offset }))
}

pub async fn stream_consume(
    _service: &FluvioServerService,
    request: Request<StreamConsumeRequest>,
) -> Result<Response<StreamConsumeStream>, Status> {
    let req = request.into_inner();
    let fluvio = Fluvio::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let partition = req.partition;
    let consumer = fluvio.partition_consumer(&req.topic, partition as u32).await.map_err(|e| Status::internal(e.to_string()))?;
    let stream = consumer.stream(Offset::absolute(req.offset).unwrap()).await.map_err(|e| Status::internal(e.to_string()))?;
    let mapped = stream.map(move |res| {
        let partition = partition;
        res.map(|record| ConsumedMessage {
            message: String::from_utf8_lossy(record.value()).to_string(),
            offset: record.offset(),
            key: String::new(),
            headers: Default::default(),
            timestamp: 0,
            message_id: String::new(),
            partition,
        }).map_err(|e| Status::internal(e.to_string()))
    });
    Ok(Response::new(Box::pin(mapped) as StreamConsumeStream))
}

pub async fn commit_offset(
    _service: &FluvioServerService,
    _request: Request<CommitOffsetRequest>,
) -> Result<Response<CommitOffsetReply>, Status> {
    Ok(Response::new(CommitOffsetReply { success: true, error: String::new() }))
}

pub async fn create_topic(
    _service: &FluvioServerService,
    request: Request<CreateTopicRequest>,
) -> Result<Response<CreateTopicReply>, Status> {
    let req = request.into_inner();
    let admin = FluvioAdmin::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let spec = TopicSpec::new_computed(req.partitions as u32, req.replication_factor as u32, None);
    let res = admin.create(req.topic, false, spec).await;
    match res {
        Ok(_) => Ok(Response::new(CreateTopicReply { success: true, error: "".to_string() })),
        Err(e) => Ok(Response::new(CreateTopicReply { success: false, error: e.to_string() })),
    }
}

pub async fn delete_topic(
    _service: &FluvioServerService,
    request: Request<DeleteTopicRequest>,
) -> Result<Response<DeleteTopicReply>, Status> {
    let req = request.into_inner();
    let admin = FluvioAdmin::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let res = admin.delete::<TopicSpec>(&req.topic).await;
    match res {
        Ok(_) => Ok(Response::new(DeleteTopicReply { success: true, error: String::new() })),
        Err(e) => Ok(Response::new(DeleteTopicReply { success: false, error: e.to_string() })),
    }
}

pub async fn list_topics(
    _service: &FluvioServerService,
    _request: Request<ListTopicsRequest>,
) -> Result<Response<ListTopicsReply>, Status> {
    let admin = FluvioAdmin::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let topics = admin.list::<TopicSpec, _>(Vec::<String>::new()).await.map_err(|e| Status::internal(e.to_string()))?;
    let names = topics.into_iter().map(|t| t.name.to_string()).collect();
    Ok(Response::new(ListTopicsReply { topics: names }))
}

pub async fn describe_topic(
    _service: &FluvioServerService,
    request: Request<DescribeTopicRequest>,
) -> Result<Response<DescribeTopicReply>, Status> {
    let req = request.into_inner();
    let admin = FluvioAdmin::connect().await.map_err(|e| Status::internal(e.to_string()))?;
    let topics = admin.list::<TopicSpec, _>(vec![req.topic.clone()]).await.map_err(|e| Status::internal(e.to_string()))?;
    if let Some(topic) = topics.into_iter().next() {
        Ok(Response::new(DescribeTopicReply {
            topic: req.topic,
            retention_ms: 0,
            config: Default::default(),
            error: String::new(),
            partitions: vec![],
        }))
    } else {
        Ok(Response::new(DescribeTopicReply {
            topic: req.topic,
            retention_ms: 0,
            config: Default::default(),
            error: "not found".to_string(),
            partitions: vec![],
        }))
    }
}

pub async fn list_consumer_groups(
    _service: &FluvioServerService,
    _request: Request<ListConsumerGroupsRequest>,
) -> Result<Response<ListConsumerGroupsReply>, Status> {
    Ok(Response::new(ListConsumerGroupsReply { groups: vec![], error: String::new() }))
}

pub async fn describe_consumer_group(
    _service: &FluvioServerService,
    _request: Request<DescribeConsumerGroupRequest>,
) -> Result<Response<DescribeConsumerGroupReply>, Status> {
    Ok(Response::new(DescribeConsumerGroupReply { group_id: String::new(), offsets: vec![], error: String::new() }))
}

pub async fn create_smart_module(
    _service: &FluvioServerService,
    _request: Request<CreateSmartModuleRequest>,
) -> Result<Response<CreateSmartModuleReply>, Status> {
    Ok(Response::new(CreateSmartModuleReply { success: true, error: String::new() }))
}

pub async fn delete_smart_module(
    _service: &FluvioServerService,
    _request: Request<DeleteSmartModuleRequest>,
) -> Result<Response<DeleteSmartModuleReply>, Status> {
    Ok(Response::new(DeleteSmartModuleReply { success: true, error: String::new() }))
}

pub async fn list_smart_modules(
    _service: &FluvioServerService,
    _request: Request<ListSmartModulesRequest>,
) -> Result<Response<ListSmartModulesReply>, Status> {
    Ok(Response::new(ListSmartModulesReply { modules: vec![], error: String::new() }))
}

pub async fn describe_smart_module(
    _service: &FluvioServerService,
    _request: Request<DescribeSmartModuleRequest>,
) -> Result<Response<DescribeSmartModuleReply>, Status> {
    Ok(Response::new(DescribeSmartModuleReply { spec: None, error: String::new() }))
}

pub async fn update_smart_module(
    _service: &FluvioServerService,
    _request: Request<UpdateSmartModuleRequest>,
) -> Result<Response<UpdateSmartModuleReply>, Status> {
    Ok(Response::new(UpdateSmartModuleReply { success: true, error: String::new() }))
}

pub async fn health_check(
    _service: &FluvioServerService,
    _request: Request<HealthCheckRequest>,
) -> Result<Response<HealthCheckReply>, Status> {
    Ok(Response::new(HealthCheckReply { ok: true, message: "ok".to_string() }))
} 