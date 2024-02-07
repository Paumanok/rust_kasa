# rust_kasa

## A minimal kasa protocol implementation in rust

This library is being written with the intention of targeting embedded devices such as the ESP32.

Not all features of the protocol will be implemented, this is currently focused on controlling and monitoring the 
Kasa HS100 power strip.

Working: 
- toggling relays by index
- setting a relay to a specific value by child_id

In progress:
- power use statistics
    - currently grabbing the realtime measurments
    - other cumulative measurements are available
