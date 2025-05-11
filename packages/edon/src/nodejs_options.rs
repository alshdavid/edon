use std::path::PathBuf;

/// Options for the Nodejs Context
///
/// [Read more here](https://nodejs.org/api/cli.html)
#[derive(Debug, Default, Clone)]
pub struct NodeOptions {
  /// Sets the max memory size of V8's old memory section. As memory consumption approaches the limit, V8 will spend more time on garbage collection in an effort to free unused memory.
  ///
  /// On a machine with 2 GiB of memory, consider setting this to 1536 (1.5 GiB) to leave some memory for other uses and avoid swapping.
  pub max_old_space_size: u32,

  /// Sets the maximum semi-space size for V8's scavenge garbage collector in MiB (mebibytes). Increasing the max size of a semi-space may improve throughput for Node.js at the cost of more memory consumption.///
  ///
  /// Since the young generation size of the V8 heap is three times (see YoungGenerationSizeFromSemiSpaceSize in V8) the size of the semi-space, an increase of 1 MiB to semi-space applies to each of the three individual semi-spaces and causes the heap size to increase by 3 MiB. The throughput improvement depends on your workload (see #42511).///  
  ///
  /// The default value depends on the memory limit. For example, on 64-bit systems with a memory limit of 512 MiB, the max size of a semi-space defaults to 1 MiB. For memory limits up to and including 2GiB, the default max size of a semi-space will be less than 16 MiB on 64-bit systems.///  
  ///
  /// To get the best configuration for your application, you should try different max-semi-space-size values when running benchmarks for your application.
  pub max_semi_space_size: u32,
  /// Set the number of threads used in libuv's threadpool to size threads.
  ///
  /// Asynchronous system APIs are used by Node.js whenever possible, but where they do not exist, libuv's threadpool is used to create asynchronous node APIs based on synchronous system APIs. Node.js APIs that use the threadpool are:
  ///
  /// all fs APIs, other than the file watcher APIs and those that are explicitly synchronous
  /// asynchronous crypto APIs such as crypto.pbkdf2(), crypto.scrypt(), crypto.randomBytes(), crypto.randomFill(), crypto.generateKeyPair()
  /// dns.lookup()
  /// all zlib APIs, other than those that are explicitly synchronous
  /// Because libuv's threadpool has a fixed size, it means that if for whatever reason any of these APIs takes a long time, other (seemingly unrelated) APIs that run in libuv's threadpool will experience degraded performance. In order to mitigate this issue, one potential solution is to increase the size of libuv's threadpool by setting the 'UV_THREADPOOL_SIZE' environment variable to a value greater than 4 (its current default value). However, setting this from inside the process using process.env.UV_THREADPOOL_SIZE=size is not guranteed to work as the threadpool would have been created as part of the runtime initialisation much before user code is run. For more information, see the libuv threadpool documentation.
  pub uv_threadpool_size: u32,
  /// Does not work because Nodejs threads cannot be debugged
  pub inspect_brk: bool,
  /// Does not work because Nodejs threads cannot be debugged
  pub inspect_port: u32,
  /// This flag will expose the gc extension from V8.
  pub expose_gc: bool,
  pub conditions: Vec<String>,
  /// When used with --build-snapshot, --snapshot-blob specifies the path where the generated snapshot blob is written to. If not specified, the generated blob is written to snapshot.blob in the current working directory.
  ///
  /// When used without --build-snapshot, --snapshot-blob specifies the path to the blob that is used to restore the application state.
  ///
  /// When loading a snapshot, Node.js checks that:
  ///  
  /// The version, architecture, and platform of the running Node.js binary are exactly the same as that of the binary that generates the snapshot.
  /// The V8 flags and CPU features are compatible with that of the binary that generates the snapshot.
  /// If they don't match, Node.js refuses to load the snapshot and exits with status code 1.
  pub snapshot_blob: PathBuf,
}
