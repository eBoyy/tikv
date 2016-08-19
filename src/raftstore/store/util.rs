// Copyright 2016 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use std::option::Option;
use std::vec::Vec;

use uuid::Uuid;
use protobuf::Message;

use kvproto::metapb;
use kvproto::eraftpb::{self, ConfChangeType};
use kvproto::raft_cmdpb::RaftCmdRequest;
use raftstore::{Result, Error};

const UUID_BYTES: usize = 16usize;

pub fn find_peer(region: &metapb::Region, store_id: u64) -> Option<&metapb::Peer> {
    for peer in region.get_peers() {
        if peer.get_store_id() == store_id {
            return Some(peer);
        }
    }

    None
}

pub fn remove_peer(region: &mut metapb::Region, store_id: u64) -> Option<metapb::Peer> {
    region.get_peers()
        .iter()
        .position(|x| x.get_store_id() == store_id)
        .map(|i| region.mut_peers().remove(i))
}

// a helper function to create peer easily.
pub fn new_peer(store_id: u64, peer_id: u64) -> metapb::Peer {
    let mut peer = metapb::Peer::new();
    peer.set_store_id(store_id);
    peer.set_id(peer_id);
    peer
}

pub fn get_uuid_from_req(cmd: &RaftCmdRequest) -> Option<Uuid> {
    Uuid::from_bytes(cmd.get_header().get_uuid())
}

pub fn encode_req(cmd: &RaftCmdRequest) -> Result<Vec<u8>> {
    let mut data = try!(cmd.write_to_bytes());
    let mut data_with_uuid = vec![];
    data_with_uuid.extend_from_slice(cmd.get_header().get_uuid());
    data_with_uuid.append(&mut data);
    Ok(data_with_uuid)
}

pub fn decode_req(bytes: &[u8]) -> (Vec<u8>, &[u8]) {
    let mut uuid = Vec::new();
    uuid.extend_from_slice(&bytes[0..UUID_BYTES]);
    (uuid, &bytes[UUID_BYTES..])
}

pub fn check_key_in_region(key: &[u8], region: &metapb::Region) -> Result<()> {
    let end_key = region.get_end_key();
    let start_key = region.get_start_key();
    if key >= start_key && (end_key.is_empty() || key < end_key) {
        Ok(())
    } else {
        Err(Error::KeyNotInRegion(key.to_vec(), region.clone()))
    }
}

pub fn conf_change_type_str(conf_type: &eraftpb::ConfChangeType) -> String {
    match *conf_type {
        ConfChangeType::AddNode => "AddNode".to_owned(),
        ConfChangeType::RemoveNode => "RemoveNode".to_owned(),
    }
}

// check whether epoch is staler than check_epoch.
pub fn is_epoch_stale(epoch: &metapb::RegionEpoch, check_epoch: &metapb::RegionEpoch) -> bool {
    epoch.get_version() < check_epoch.get_version() ||
    epoch.get_conf_ver() < check_epoch.get_conf_ver()
}

#[cfg(test)]
mod tests {
    use kvproto::metapb;

    use super::*;

    #[test]
    fn test_peer() {
        let mut region = metapb::Region::new();
        region.set_id(1);
        region.mut_peers().push(new_peer(1, 1));

        assert!(find_peer(&region, 1).is_some());
        assert!(find_peer(&region, 10).is_none());

        assert!(remove_peer(&mut region, 1).is_some());
        assert!(remove_peer(&mut region, 1).is_none());
        assert!(find_peer(&region, 1).is_none());

    }
}
