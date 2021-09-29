use crate::mqtt::message::v3::{PublishMessage, ConnectMessage};
use crate::mqtt::message::{BaseConnect, BaseMessage};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::mqtt::tools::protocol::{MqttProtocolLevel, MqttWillFlag, MqttQos, MqttRetain, MqttDup};
use crate::mqtt::message::{MqttMessageKind, MqttMessage, MqttBytesMessage};
use crate::mqtt::tools::types::TypeKind;
use crate::mqtt::v3_handle;
use log::{debug, error};

#[derive(Debug, Clone, Eq, Hash)]
pub struct ClientID(pub String);

impl ClientID {
    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

impl AsRef<ClientID> for ClientID {
    fn as_ref(&self) -> &ClientID {
        &self
    }
}

impl From<String> for ClientID {
    fn from(s: String) -> Self {
        ClientID(s)
    }
}

impl From<&str> for ClientID {
    fn from(s: &str) -> Self {
        ClientID(s.to_owned())
    }
}

impl From<&ClientID> for ClientID {
    fn from(client: &ClientID) -> Self {
        client.to_owned()
    }
}

impl PartialEq for ClientID {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        PartialEq::ne(&self.0, &other.0)
    }
}


#[derive(Debug, Clone)]
pub enum TopicMessage {
    ContentV3(ClientID, crate::mqtt::message::v3::PublishMessage),
    ContentV5(ClientID, crate::mqtt::message::v5::PublishMessage),
    Will(PublishMessage),
}

impl TopicMessage {
    pub fn get_topic(&self) -> Option<&String> {
        match self {
            TopicMessage::ContentV3(_, msg) => { Some(&msg.topic) }
            TopicMessage::ContentV5(_, msg) => { Some(&msg.topic) }
            TopicMessage::Will(_) => { None }
        }
    }
}

pub struct Subscript {
    container: Arc<Mutex<HashMap<String, Topic>>>,
}

impl Subscript {
    pub fn new() -> Subscript {
        Subscript { container: Arc::new(Mutex::new(HashMap::default())) }
    }

    pub async fn contain<S: AsRef<str>>(&self, topic_name: S) -> bool {
        self.container.lock().await.contains_key(topic_name.as_ref())
    }

    pub async fn len(&self) -> usize {
        self.container.lock().await.len()
    }

    pub async fn add<S: Into<String>>(&self, topic_name: S, topic: Topic) -> Option<Topic> {
        self.container.lock().await.insert(topic_name.into(), topic)
    }

    pub async fn remove<S: AsRef<str>>(&self, topic_name: S) -> Option<Topic> {
        self.container.lock().await.remove(topic_name.as_ref())
    }

    pub async fn is_subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) -> bool {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().contain(client_id)
    }

    pub async fn new_subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS, sender: Sender<LineMessage>) {
        let mut top = Topic::new(topic_name.as_ref());
        top.subscript(client_id.as_ref(), sender);
        self.add(topic_name.as_ref(), top).await;
    }

    pub fn subscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS, sender: Sender<LineMessage>) {
        match self.container.try_lock() {
            Ok(mut container) => {
                if let Some(t) = container.get_mut(topic_name.as_ref()) {
                    t.subscript(client_id.as_ref(), sender);
                }
            }
            Err(e) => {
                error!("{:?}", e)
            }
        }
    }

    pub async fn unsubscript<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) {
        self.container.lock().await.get_mut(topic_name.as_ref()).unwrap().unsubscript(client_id);
    }

    pub async fn exit<S: AsRef<ClientID>>(&self, client_id: S) {
        for (_, topic) in self.container.lock().await.iter_mut() {
            topic.unsubscript(client_id.as_ref());
        }
    }

    pub async fn topics(&self) -> Vec<String> {
        self.container.lock().await.keys().cloned().collect::<Vec<String>>()
    }

    pub async fn clients<S: AsRef<str>>(&self, topic_name: S) -> Vec<ClientID> {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().client_id_list()
    }

    pub async fn client_len<S: AsRef<str>>(&self, topic_name: S) -> usize {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().client_len()
    }

    pub async fn broadcast<S: AsRef<str>>(&self, topic_name: S, msg: &TopicMessage) {
        if let Some(t) = self.container.lock().await.get(topic_name.as_ref()) {
            t.broadcast(msg).await
        }
    }

    pub async fn get_client<S: AsRef<str>, SS: AsRef<ClientID>>(&self, topic_name: S, client_id: SS) -> Sender<LineMessage> {
        self.container.lock().await.get(topic_name.as_ref()).unwrap().senders.get(client_id.as_ref()).unwrap().clone()
    }
}

#[derive(Debug)]
pub struct Topic {
    name: String,
    senders: HashMap<ClientID, Sender<LineMessage>>,
}

impl Topic {
    pub fn new<S: Into<String>>(name: S) -> Topic {
        Topic { name: name.into(), senders: HashMap::new() }
    }
}

impl Topic {
    pub fn subscript<S: Into<ClientID>>(&mut self, client_id: S, sender: Sender<LineMessage>) {
        let id = client_id.into();
        debug!("subscript client id: {:?}", &id);
        self.senders.insert(id, sender);
    }

    pub fn unsubscript<S: AsRef<ClientID>>(&mut self, client_id: S) -> Option<Sender<LineMessage>> {
        if self.senders.contains_key(client_id.as_ref()) {
            return self.senders.remove(client_id.as_ref());
        }
        None
    }

    pub fn client_id_list(&self) -> Vec<ClientID> {
        self.senders.keys().cloned().collect::<Vec<ClientID>>()
    }

    pub fn client_len(&self) -> usize {
        self.senders.len()
    }

    pub async fn broadcast(&self, msg: &TopicMessage) {
        for (_, sender) in self.senders.iter() {
            sender.send(LineMessage::SubscriptionMessage(msg.clone())).await;
        }
    }

    pub fn contain<S: AsRef<ClientID>>(&self, client_id: S) -> bool {
        self.senders.contains_key(client_id.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum LineMessage {
    SocketMessage(Vec<u8>),
    SubscriptionMessage(TopicMessage),
}

pub struct Line {
    sender: Sender<LineMessage>,
    receiver: Receiver<LineMessage>,
    client_id: Option<ClientID>,
    protocol_name: Option<String>,
    protocol_level: Option<MqttProtocolLevel>,
    will_flag: Option<MqttWillFlag>,
    will_qos: Option<MqttQos>,
    will_retain: Option<MqttRetain>,
    will_topic: Option<String>,
    will_message: Option<String>,
}

impl Line {
    pub fn new() -> Line {
        let (sender, receiver) = mpsc::channel(128);
        Line {
            sender,
            receiver,
            client_id: None,
            protocol_name: None,
            protocol_level: None,
            will_flag: None,
            will_qos: None,
            will_retain: None,
            will_topic: None,
            will_message: None,
        }
    }

    pub fn get_client_id(&self) -> &ClientID {
        self.client_id.as_ref().unwrap()
    }

    pub fn init_protocol(&mut self, protocol_name: String, protocol_level: MqttProtocolLevel) {
        self.protocol_name = Some(protocol_name);
        self.protocol_level = Some(protocol_level);
    }

    pub fn is_will_flag(&self) -> bool {
        self.will_flag.unwrap() == MqttWillFlag::Enable
    }

    pub fn get_v3_topic_message(&self) -> TopicMessage {
        let msg = PublishMessage::new(
            self.will_qos.unwrap(),
            MqttDup::Disable,
            self.will_retain.unwrap(),
            self.will_topic.as_ref().unwrap().to_owned(),
            0,
            self.will_message.as_ref().unwrap().to_owned(),
        );
        TopicMessage::ContentV3(self.get_client_id().clone(), msg)
    }

    pub fn get_will_topic(&self) -> &String {
        self.will_topic.as_ref().unwrap()
    }

    pub fn init_v3(&mut self, connect_msg: &ConnectMessage) {
        self.client_id = Some(ClientID(connect_msg.payload.client_id.to_owned()));
        self.will_flag = Some(connect_msg.will_flag);
        self.will_qos = Some(connect_msg.will_qos);
        self.will_retain = Some(connect_msg.will_retain);
        self.will_topic = connect_msg.payload.will_topic.clone();
        self.will_message = connect_msg.payload.will_message.clone();
    }

    pub fn init_v5(&mut self, connect_msg: &crate::mqtt::message::v5::ConnectMessage) {
        self.client_id = Some(ClientID(connect_msg.payload.client_id.to_owned()));
        self.will_flag = Some(connect_msg.will_flag);
        self.will_qos = Some(connect_msg.will_qos);
        self.will_retain = Some(connect_msg.will_retain);
        self.will_topic = connect_msg.payload.will_topic.clone();
        self.will_message = connect_msg.payload.will_message.clone();
    }

    pub fn get_sender(&self) -> Sender<LineMessage> {
        self.sender.clone()
    }

    pub async fn recv(&mut self) -> Option<MqttMessageKind> {
        match self.receiver.recv().await {
            None => { None }
            Some(msg) => {
                match msg {
                    LineMessage::SocketMessage(msg) => self.handle_socket_message(msg).await,
                    LineMessage::SubscriptionMessage(msg) => self.handle_subscription_message(msg)
                }
            }
        }
    }

    async fn handle_socket_message(&mut self, msg: Vec<u8>) -> Option<MqttMessageKind> {
        let base_msg = BaseMessage::from(msg);
        if base_msg.get_message_type() == TypeKind::CONNECT {
            let connect = BaseConnect::from(&base_msg);
            self.init_protocol(connect.get_protocol_name(), connect.get_protocol_level());
        }

        if let Some(level) = self.protocol_level {
            return match level {
                MqttProtocolLevel::Level3_1_1 => v3_handle::match_v3_data(self, base_msg).await,
                _ => { return None; }
            };
        }
        None
    }

    fn handle_subscription_message(&mut self, msg: TopicMessage) -> Option<MqttMessageKind> {
        return match msg {
            TopicMessage::ContentV3(from_id, content) => {
                debug!("from: {:?}", from_id);
                debug!("to: {:?}", self.get_client_id());
                if self.get_client_id() != &from_id {
                    return Some(MqttMessageKind::Response(content.as_bytes().to_vec()));
                }
                None
            }
            TopicMessage::ContentV5(from_id, content) => {
                debug!("from: {:?}", from_id);
                debug!("to: {:?}", self.get_client_id());
                if self.get_client_id() != &from_id {
                    return Some(MqttMessageKind::Response(content.as_bytes().to_vec()));
                }
                None
            }
            TopicMessage::Will(content) => {
                Some(MqttMessageKind::Response(content.as_bytes().to_vec()))
            }
        };
    }
}


