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
