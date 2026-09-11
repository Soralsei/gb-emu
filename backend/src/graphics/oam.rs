#![allow(dead_code)]

enum OamTransferState {
    Idle,
    Queued,
    InProgress,
}

pub struct OAMController {
    oam_transfer_state: OamTransferState,
}
