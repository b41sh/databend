// Copyright 2021 Datafuse Labs.
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

mod meta_metrics;

pub use meta_metrics::incr_meta_metrics_applying_snapshot;
pub use meta_metrics::incr_meta_metrics_leader_change;
pub use meta_metrics::incr_meta_metrics_proposals_failed;
pub use meta_metrics::incr_meta_metrics_proposals_pending;
pub use meta_metrics::incr_meta_metrics_read_failed;
pub use meta_metrics::incr_meta_metrics_watchers;
pub use meta_metrics::init_meta_metrics_recorder;
pub use meta_metrics::meta_metrics_to_prometheus_string;
pub use meta_metrics::set_meta_metrics_has_leader;
pub use meta_metrics::set_meta_metrics_is_leader;
pub use meta_metrics::set_meta_metrics_proposals_applied;
