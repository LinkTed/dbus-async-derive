#[macro_use]
extern crate log;
#[macro_use]
extern crate dbus_async;
use dbus_async::{Handle, DBus};
use dbus_message_parser::Message;



struct Test {
    dbus: DBus,
}


impl Handle for Test {
    fn handle(&mut self, msg: Message) {
        // Try to get the Interface
        match get_interface_from_msg!(self.dbus, msg).as_ref() {
            "test.interface" => {
                // Try to get the Member
                match get_member_from_msg!(self.dbus, msg).as_ref() {
                    "TestMethod1" => {
                        // Check if the signature correct
                        // There should be String and Uint32 given
                        check_signature_from_msg!(self.dbus, msg, "su");
                        let body = msg.get_body();
                        // Parse the argument of the method
                        let arg0 = get_arg_from_vec!(self.dbus, msg, body, 0, String);
                        let arg1 = get_arg_from_vec!(self.dbus, msg, body, 1, Uint32);
                        // The code of the method of TestMethod1
                        // ...
                    }
                    "TestMethod2" => {
                        // Check if the signature correct
                        // There should no argument given
                        check_if_no_signature_from_msg!(self.dbus, msg);
                        // The code of the method of TestMethod2
                        // ...
                    }
                    _ => unknown_member!(self.dbus, msg)
                }
            }
            _ => unknown_interface!(self.dbus, msg)
        }
    }
}
