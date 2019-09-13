use std::prelude::v1::*;

use super::ResourceId;
use super::TaskConfig;
use super::TaskWithFilter;

pub struct TaskStage {
    combined_reads: Vec<ResourceId>, //TODO: Consider changing to a set
    combined_writes: Vec<ResourceId>, //TODO: Consider changing to a set
    any_reads_all: bool,
    any_writes_all: bool,
    tasks: Vec<TaskWithFilter>
}

impl TaskStage {
    pub fn new() -> Self {
        TaskStage {
            combined_reads: vec![],
            combined_writes: vec![],
            any_reads_all: false,
            any_writes_all: false,
            tasks: vec![]
        }
    }

    pub fn can_add_task(&self, new_task: &TaskConfig) -> bool {
        // the new task writes all => we can only accept if there are no existing reads/writes
        if new_task.write_all {
            return self.combined_reads.is_empty() &&
                self.combined_writes.is_empty() &&
                !self.any_reads_all &&
                !self.any_writes_all;
        }

        // existing task writes all => we can only accept if the new task has no writes/reads
        if self.any_writes_all {
            return new_task.reads.is_empty() &&
                new_task.writes.is_empty() &&
                !new_task.read_all &&
                !new_task.write_all;
        }

        // new task reads all => we're find as long as we aren't writing anything
        if new_task.read_all {
            debug_assert!(!self.any_writes_all);
            return self.combined_writes.is_empty()
        }

        // any existing task reads all => we can't accept anything that writes
        if self.any_reads_all {
            debug_assert!(!new_task.write_all);
            return new_task.writes.is_empty()
        }

        // Verify the new task isn't writing anything that is being read/write
        for write in &new_task.writes {
            if self.combined_writes.contains(write) || self.combined_reads.contains(write) {
                return false;
            }
        }

        // Verify the new task isn't reading anything that is already being written
        for write in &self.combined_writes {
            if new_task.reads.contains(write) {
                return false;
            }
        }

        true
    }

    pub fn add_task(&mut self, new_task: TaskConfig) {
        debug_assert!(self.can_add_task(&new_task));

        self.any_reads_all |= new_task.read_all;
        self.any_writes_all |= new_task.write_all;

        for read in &new_task.reads {
            if !self.combined_reads.contains(read) {
                self.combined_reads.push(read.clone());
            }
        }

        for write in &new_task.reads {
            if !self.combined_writes.contains(write) {
                self.combined_writes.push(write.clone());
            }
        }

        let task_with_filter = TaskWithFilter::new(new_task);

        self.tasks.push(task_with_filter);
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn tasks(&self) -> &Vec<TaskWithFilter> {
        &self.tasks
    }
}

