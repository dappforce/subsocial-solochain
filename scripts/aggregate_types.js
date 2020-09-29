// This script reads the type definitions from all pallets and the runtime's own types.
// Aggregates types and writes them to a single file "types.json" in the root of this repo.

const fs = require('fs');
const path = require('path');

const pallets = [
  "donations",
  "faucet",
  "moderation",
  "permissions",
  "post-history",
  "posts",
  "profile-follows",
  "profile-history",
  "profiles",
  "reactions",
  "roles",
  "scores",
  "session-keys",
  "space-follows",
  "space-history",
  "space-ownership",
  "spaces",
  "subscriptions",
  "utils",
]

// Types that are native to the runtime itself (i.e. come from lib.rs)
// These specifics are from https://polkadot.js.org/api/start/types.extend.html#impact-on-extrinsics
const runtimeTypeOverrides = {
  "Address": "AccountId",
  "LookupSource": "AccountId"
}

let allTypes = {
  ...runtimeTypeOverrides,
  "IpfsCid": "Text"
};

// Aggregate types from all pallets into `allTypes`.
for (let pallet of pallets) {
  let jsonPath = path.join(__dirname, `../pallets/${pallet}/types.json`);
  let palletTypes = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));
  allTypes = {...allTypes, ...palletTypes};
}

// Write aggregated types into a single file:
fs.writeFileSync(
  path.join(__dirname, "../types.json"),
  JSON.stringify(allTypes, null, 2),
  'utf8'
);
