<!--
Copyright 2020 - Nym Technologies SA <contact@nymtech.net>
SPDX-License-Identifier: Apache-2.0
-->

# Nym Tauri Wallet Webdriver testsuite

A webdriverio test suite implementation using tauri driver 
with a page object model design. This project is to provide quick iterative feedback
on the UI of the tauri nym wallet.

Currently, tauri-driver is available to run on Windows and Linux machines.

## Installation prerequisites 
* `Yarn`
* `NodeJS >= v16.8.0`
* `Rust & cargo >= v1.51`
* `tauri-driver`


## Key Information
* Please read the instructions on the README.md on the tauri-project on how to build the application
* Be on an OS of your choice which is capable of launching tauri driver - (for more information please visit https://tauri.studio/en/docs/usage/guides/webdriver/introduction
this will specify the additional drivers you need if you do need them)
* The path to run the application is set in the `wdio.conf.js` which lives in the root directory 
* Before running the suite you need to build the application and check that the application has
built successfully, if so, you will have an executable sitting in the target directory in `src-tauri/*/nym_wallet` (refer to point 1)
* The suite will not be able to detect elements on screen if you select a release build, however you can run tests against a release target


## Installation & usage
*  `test excution happens inside /webdriver directory`
*  `test data needs to be provided inside the user-data.js module`
```
example: 
//mnemonic is a base64 enconded value, which is your 24 character passphrase, these values are for illustration purposes
      {     
      "mnemonic" : "dGhpcyBpcyBhIHBhc3NwaHJhc2UK",   
      "punk_address" : "punk1f3dzkhmunma5ze5q952daxca6371989189",    
      "receiver_address" : "punk1p0ce82jxxglpmutvhq4mdwgcwf4avm5n1821982",    
      "amount_to_send" : "5",
      }
```
*  `yarn test` - the first test run will take some time to spin up be patient

## Updates
Disclaimer: As this project is WIP, there's a lot due to be updated and modified. This is to get the project up and running.
Refactoring needs to take place in certain areas to reduce code duplication.
Currently this project has been dev'd against a Linux based OS, not currently trialled and tested against Windows.