const os = require("os");
const path = require("path");
const { spawn, spawnSync } = require("child_process");
const nym_path = "../target/release/tauri-wallet";

// keep track of the `tauri-driver` child process
let tauriDriver;

exports.config = {
  //run sequentially, as using one default user may cause issues
  specs: [
    "./tests/specs/test.wallet.home.js",
    "./tests/specs/test.wallet.send.js",
    "./tests/specs/test.wallet.receive.js",
    "./tests/specs/test.wallet.bond.js",
    "./tests/specs/test.wallet.delegate.js"],
  
   //run tests by providing --suite {{login}} 
  suites: {
    login: ["./tests/specs/test.wallet.home.js"],
    sendAndReceive: ["./tests/specs/test.wallet.send.js", 
                    "./tests/specs/test.wallet.receive.js"],
    bond: ["./tests/specs/test.wallet.bond.js"],
    delegate: ["./tests/specs/test.wallet.delegate.js"]
  },
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: nym_path,
        browser: "wry"
      },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },
  logLevel: 'debug',

  // this is documentented in the readme - you will need to build the project first
  // ensure the rust project is built since we expect this binary to exist for the webdriver sessions
  //onPrepare: () => spawnSync("cargo", ["build", "--release"]),

  // ensure we are running `tauri-driver` before the session starts so that we can proxy the webdriver requests
  beforeSession: () =>
  (tauriDriver = spawn(
    path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver"),
    [],
    { stdio: [null, process.stdout, process.stderr] }
  )),

  // clean up the `tauri-driver` process we spawned at the start of the session
  afterSession: () => tauriDriver.kill()
};