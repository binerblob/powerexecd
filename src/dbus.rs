pub mod upower {
    use zbus::dbus_proxy;
    use zbus::zvariant::OwnedValue;
    
    #[derive(
        Copy,
        Clone,
        Debug,
        PartialEq,
        Eq,
        Hash,
        OwnedValue
    )]
    pub enum BatteryState {
        Unknown = 0,
        Charging = 1,
        Discharging = 2,
        Empty = 3,
        FullyCharged = 4,
        PendingCharge = 5,
        PendingDischarge = 6,
    }
    
    #[dbus_proxy(
        interface = "org.freedesktop.UPower.Device",
        default_service = "org.freedesktop.UPower",
        assume_defaults = false
    )]
    trait Device {
        #[dbus_proxy(property)]
        fn percentage(&self) -> zbus::Result<f64>;
        
        #[dbus_proxy(property)]
        fn state(&self) -> zbus::Result<BatteryState>;
    }
}

pub mod logind {
    use zbus::dbus_proxy;

    #[dbus_proxy(
        interface = "org.freedesktop.login1.Manager",
        default_service = "org.freedesktop.login1",
        default_path = "/org/freedesktop/login1"
    )]
    trait Manager {
        #[dbus_proxy(signal)]
        fn session_removed(&self, id: String, object_path: zbus::zvariant::OwnedObjectPath) -> zbus::Result<()>;

        #[dbus_proxy(signal)]
        fn session_new(&self, id: String, object_path: zbus::zvariant::OwnedObjectPath) -> zbus::Result<()>;

        #[dbus_proxy(signal)]
        fn active_session(&self) -> crate::zbus::Result<(String, crate::zvariant::OwnedObjectPath)>;
        
        fn get_session_by_pid(&self, pid: u32) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

        fn get_session(&self, session_id: String) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    }
}
