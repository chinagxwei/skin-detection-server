use crate::mqtt::v3_server::{Line, TopicMessage, ClientID};
use crate::mqtt::message::{BaseMessage, MqttMessageKind, v3};
use crate::mqtt::message::v3::{MqttMessageV3, ConnackMessage, PublishMessage, PubackMessage, SubscribeMessage, UnsubscribeMessage, UnsubackMessage, DisconnectMessage, SubackMessage, PubrelMessage};
use crate::mqtt::tools::protocol::MqttQos;
use crate::{SUBSCRIPT, MACHINE_CONTAINER, MachineID, Machine, MachineStatus};
use log::{debug, info};
use crate::http::MachineMessage;

pub async fn match_v3_data(line: &mut Line, base_msg: BaseMessage) -> Option<MqttMessageKind> {
    if let Some(v3) = MqttMessageKind::v3(base_msg) {
        return match (
            v3.is_v3(),
            handle_v3(line, v3.get_v3()).await,
            v3.is_v3s(),
            v3.get_v3s()
        ) {
            (true, Some(res_msg), _, _) => {
                if res_msg.is_disconnect() {
                    Some(MqttMessageKind::Exit(res_msg.as_bytes().to_vec()))
                } else {
                    Some(MqttMessageKind::Response(res_msg.as_bytes().to_vec()))
                }
            }
            (_, _, true, Some(items)) => {
                let mut res = vec![];
                for x in items {
                    if let Some(res_msg) = handle_v3(line, Some(x)).await {
                        res.push(res_msg.as_bytes().to_vec());
                    }
                }
                Some(MqttMessageKind::Response(res.concat()))
            }
            _ => None
        };
    }
    None
}

async fn send_qrcdoe(id: String, qrcode_url: String) {
    let machine_message = MachineMessage::qrcode(id.clone(),qrcode_url);
    let topic = format!("{}-topic", id);
    let publish_message = v3::PublishMessage::simple_new_msg(
        topic,
        0,
        serde_json::to_string(&machine_message).unwrap(),
    );
    let topic_msg = TopicMessage::ContentV3(ClientID("idreamspace-server".to_string()), publish_message);
    if let Some(topic) = topic_msg.get_topic() {
        SUBSCRIPT.broadcast(topic, &topic_msg).await;
    }
}

async fn handle_v3(line: &mut Line, kind_opt: Option<&MqttMessageV3>) -> Option<MqttMessageV3> {
    if let Some(kind) = kind_opt {
        match kind {
            MqttMessageV3::Connect(msg) => {
                let machine_id = MachineID(msg.payload.client_id.clone());
                if let Some(url) = MACHINE_CONTAINER.get_qrcode(&machine_id).await {
                    send_qrcdoe(msg.payload.client_id.clone(), url).await;
                }
                MACHINE_CONTAINER.append(machine_id, Machine {
                    id: msg.payload.client_id.clone(),
                    qrcode_url: "".to_string(),
                    status: MachineStatus::Online,
                }).await;
                line.init_v3(msg);
                return Some(MqttMessageV3::Connack(ConnackMessage::default()));
            }
            // MqttMessageV3::Puback(msg) => {
            // }
            MqttMessageV3::Subscribe(msg) => return handle_v3_subscribe(line, msg).await,
            MqttMessageV3::Unsubscribe(msg) => return handle_v3_unsubscribe(line, msg).await,
            MqttMessageV3::Publish(msg) => return handle_v3_publish(line, msg).await,
            MqttMessageV3::Pingresp(msg) => return Some(MqttMessageV3::Pingresp(msg.clone())),
            MqttMessageV3::Disconnect(_) => return handle_v3_disconnect(line).await,
            MqttMessageV3::Pubrec(msg) => return Some(MqttMessageV3::Pubrel(PubrelMessage::new(msg.message_id))),
            _ => { return None; }
        }
    }
    None
}

async fn handle_v3_publish(line: &mut Line, msg: &PublishMessage) -> Option<MqttMessageV3> {
    let topic_msg = TopicMessage::ContentV3(line.get_client_id().to_owned(), msg.clone());
    debug!("topic: {:?}", topic_msg);
    SUBSCRIPT.broadcast(&msg.topic, &topic_msg).await;
    if msg.qos == MqttQos::Qos1 {
        return Some(MqttMessageV3::Puback(PubackMessage::new(msg.message_id)));
    } else if msg.qos == MqttQos::Qos2 {}
    return None;
}

async fn handle_v3_subscribe(line: &mut Line, msg: &SubscribeMessage) -> Option<MqttMessageV3> {
    debug!("{:?}", msg);
    let topic = &msg.topic;
    if SUBSCRIPT.contain(topic).await {
        SUBSCRIPT.subscript(topic, line.get_client_id(), line.get_sender());
    } else {
        SUBSCRIPT.new_subscript(topic, line.get_client_id(), line.get_sender()).await;
    }
    debug!("broadcast topic len: {}", SUBSCRIPT.len().await);
    debug!("broadcast topic list: {:?}", SUBSCRIPT.topics().await);
    debug!("broadcast client len: {:?}", SUBSCRIPT.client_len(topic).await);
    debug!("broadcast client list: {:?}", SUBSCRIPT.clients(topic).await);
    let sm = SubackMessage::from(msg.clone());
    debug!("{:?}", sm);
    return Some(MqttMessageV3::Suback(sm));
}

async fn handle_v3_unsubscribe(line: &mut Line, msg: &UnsubscribeMessage) -> Option<MqttMessageV3> {
    debug!("topic name: {}", &msg.topic);
    if SUBSCRIPT.contain(&msg.topic).await {
        if SUBSCRIPT.is_subscript(&msg.topic, line.get_client_id()).await {
            SUBSCRIPT.unsubscript(&msg.topic, line.get_client_id()).await;
            return Some(MqttMessageV3::Unsuback(UnsubackMessage::new(msg.message_id)));
        }
    }
    return None;
}

async fn handle_v3_disconnect(line: &mut Line) -> Option<MqttMessageV3> {
    info!("client disconnect");
    if line.is_will_flag() {
        let topic_msg = line.get_v3_topic_message();
        SUBSCRIPT.broadcast(line.get_will_topic(), &topic_msg).await;
    }
    SUBSCRIPT.exit(line.get_client_id()).await;
    let id = MachineID(line.get_client_id().as_string());
    MACHINE_CONTAINER.remove(&id).await;
    return Some(MqttMessageV3::Disconnect(DisconnectMessage::default()));
}
