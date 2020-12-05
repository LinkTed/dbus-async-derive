use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::{Error, MessageHeader};
use std::convert::TryInto;

#[derive(Handler)]
#[interface("org.example.deleteable", method("Delete", delete))]
struct DeletableObject {}

impl DeletableObject {
    async fn delete(
        &mut self,
        dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<(), (Error, String)> {
        // Caution: This will remove the object from the list
        //          (message which are already processed will be handle)
        dbus.delete_object_path("/org/example/deleteable".try_into().unwrap())
            .expect("Could not delete the object");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (dbus, _join_handle) = DBus::session(true)
        .await
        .expect("Failed to get the DBus object");

    let deleteable_object = DeletableObject {};
    deleteable_object
        .bind(dbus, "/org/example/deleteable".try_into().unwrap())
        .await
        .expect("Object was not deleted successfully");
}
