use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::{Error, MessageHeader};
use std::convert::TryInto;

#[derive(Handler)]
#[interface(
    "org.example.properties",
    // Read and write property of type string
    property(
        // The name of the property
        "StringProperty",
        // The DBus type of the property
        "s",
        // The get function to retrieve the current value
        get_string_property = "get",
        // The set function to change the value
        set_string_property = "set"
    ),
    // Read only property of type dict<i32, String>
    property(
        // The name of the property
        "DictProperty",
        // The DBus type of the property
        "a{is}",
        // The get function to retrieve the current value
        get_dict_property = "get",
    ),
    // Write only property of type i32
    property(
        // The name of the property
        "IntProperty",
        // The DBus type of the property
        "i",
        // The set function to change the value
        set_int_property = "set",
    ),
)]
struct PropertiesObject {
    string_property: String,
    dict_property: Vec<(i32, String)>,
    int_property: i32,
}

impl PropertiesObject {
    fn new() -> PropertiesObject {
        let mut dict_property = Vec::new();
        dict_property.push((1, "TEST".to_string()));
        dict_property.push((2, "EXAMPLE".to_string()));

        PropertiesObject {
            string_property: "Init value".to_string(),
            dict_property,
            int_property: 0,
        }
    }

    async fn get_string_property(
        &mut self,
        _dbus: &DBus,
        msg_header: &MessageHeader,
    ) -> Result<String, (Error, String)> {
        // The code of the get function of the StringProperty property
        // Only message which have a sender can access to this property
        if let Some(_) = msg_header.get_sender() {
            Ok(self.string_property.clone())
        } else {
            Err((
                "org.freedesktop.DBus.Properties.Error".try_into().unwrap(),
                "This is an error message".to_string(),
            ))
        }
    }

    async fn set_string_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        new_value: String,
    ) -> Result<(), (Error, String)> {
        // The code of the set function of the StringProperty property
        // If the string is empty then do not change the value
        if new_value.is_empty() {
            Err((
                "org.freedesktop.DBus.Properties.Error".try_into().unwrap(),
                "This is an error message".to_string(),
            ))
        } else {
            self.string_property = new_value;
            Ok(())
        }
    }

    async fn get_dict_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<Vec<(i32, String)>, (Error, String)> {
        Ok(self.dict_property.clone())
    }

    async fn set_int_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        new_value: i32,
    ) -> Result<(), (Error, String)> {
        self.int_property = new_value;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (dbus, _connection_join_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    let object_path = "/org/example/properties".try_into().unwrap();
    let property_object = PropertiesObject::new();
    property_object
        .bind(dbus, object_path)
        .await
        .expect("Something went wrong");
}
