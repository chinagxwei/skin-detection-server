use crate::mqtt::hex::{PropertyItem, Property};
use std::convert::TryFrom;
use log::{ error};

pub fn connect(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_connect_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }
    properties
}

pub fn connack(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_connack_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn publish(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_publish_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn subscribe(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_subscribe_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn unsubscribe(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_unsubscribe_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn suback(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_pub_and_sub_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn unsuback(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_pub_and_sub_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn disconnect(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_disconnect_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn auth(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_auth_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn pub_and_sub(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_pub_and_sub_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}

pub fn will_properties(mut length: u32, mut data: &[u8]) -> Vec<PropertyItem> {
    let mut properties = vec![];
    loop {
        let property = data[0];
        match Property::try_from(property) {
            Ok(p) => {
                if p.is_will_property() {
                    if let Some((item, last_data)) = p.unpack_property_handle(&mut length, data.get(1..).unwrap()) {
                        data = last_data;
                        properties.push(item);
                    }
                }
            }
            Err(e) => {
                error!("Property {:?} not exist", e)
            }
        }

        if length <= 0 {
            break;
        }
    }

    properties
}
