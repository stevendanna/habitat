// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A collection of handlers for the JobSrv dispatcher

use dbcache::{self, ExpiringSet, IndexSet, InstaSet};
use hab_net::server::Envelope;
use protocol::net::{self, ErrCode};
use protocol::jobsrv as proto;
use zmq;

use super::ServerState;
use error::Result;

pub fn job_create(req: &mut Envelope,
                  sock: &mut zmq::Socket,
                  state: &mut ServerState)
                  -> Result<()> {
    let mut job = proto::Job::new();
    job.set_state(proto::JobState::default());
    state.datastore.jobs.write(&mut job).unwrap();
    state.datastore.job_queue.enqueue(&job).unwrap();
    try!(state.work_manager.notify_work());
    try!(req.reply_complete(sock, &job));
    Ok(())
}

pub fn job_get(req: &mut Envelope, sock: &mut zmq::Socket, state: &mut ServerState) -> Result<()> {
    let msg: proto::JobGet = try!(req.parse_msg());
    match state.datastore.jobs.find(&msg.get_id()) {
        Ok(job) => {
            let reply: proto::Job = job.into();
            try!(req.reply_complete(sock, &reply));
        }
        Err(dbcache::Error::EntityNotFound) => {
            let err = net::err(ErrCode::ENTITY_NOT_FOUND, "jb:job-get:1");
            try!(req.reply_complete(sock, &err));
        }
        Err(e) => {
            error!("datastore error, err={:?}", e);
            let err = net::err(ErrCode::INTERNAL, "jb:job-get:2");
            try!(req.reply_complete(sock, &err));
        }
    }
    Ok(())
}
