# Polaris

Lightweight P2P library for local networks, optimized for constrained embedded systems. Uses UDP to quickly transfer data from nodes to the main controller. Designed for centralized systems with many peripheral devices communicating to a head device.


## Quick Start
**Note:** Examples are expected future API, not implemented yet.

**Controller:**
```rust
let controller = Controller::new(20);
controller.on(
    "temperature",
    |node, payload| { println!("Temp: {} °C\nFrom Device {}", payload, node.id()) });

controller.start().await;
```

**Node:**
```rust
let node = Node::new(DeviceId::new(5));
loop {
    if node.discover() == NodeState::Found {
        break
    }

loop {
    let data = sensor.collect();
    node.send(data);
    sleep(10000);
}
```

## Roadmap
- [x] Core types
- [] UDP Transport
- [] Discovery
- [] Security\Encryption
- [] Node-to-Node connections

## License
This project is licensed under the Apache License 2.0
