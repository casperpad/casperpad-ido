import { Keys } from "casper-js-sdk";
const { Ed25519 } = Keys;
import { DEFAULT_TTL } from "casper-js-client-helper/dist/constants";

const DEPLOY_GAS_PAYMENT_FOR_SESSION_TRANSFER = 3000000000;
const DEPLOY_GAS_PAYMENT_FOR_INSTALL = 150000000000;
const publicKey = Ed25519.readBase64WithPEM(
  "MCowBQYDK2VwAyEACudgC5VEkWJLooc25oCe0rNwULChYs8J2Pu2NDMKnK4="
);
// Ed25519.privateToPublicKey(privateKey)
const privateKey = Ed25519.readBase64WithPEM(
  "MC4CAQAwBQYDK2VwBCIEIFqWS7LqIyeGwh/6g50yFWuAf+M4laQ17aZ+x4h8U/Pk"
);

const LOCAL_WALLET_PUBLIC = Ed25519.readBase64WithPEM("MCowBQYDK2VwAyEAhPbSYPTuaGnds2r/4VRW3mrgRSePovRnu2d1Yc4NrVU=");
const LOCAL_WALLET_PRIVATE = Ed25519.readBase64WithPEM("MC4CAQAwBQYDK2VwBCIEIHRZr1HEgKVbgchuatwA7dCWDWB7QZe+bpDb5dguIyLE");




const MASTER_WALLET_KEYPAIR = Ed25519.loadKeyPairFromPrivateFile("/home/master/pitzerbert_secret_key.pem");
const DEPLOY_CHAIN_NAME = "casper-test";
const DEPLOY_NODE_ADDRESS = "http://95.217.34.115:7777/rpc";


const DEPLOY_NODE_EVENT_STREAM_ADDRESS = "http://localhost:18101/events/main";

// const DEPLOY_CHAIN_NAME = "casper-net-1";
// const DEPLOY_NODE_ADDRESS = "http://localhost:11101/rpc";
// const MASTER_WALLET_KEYPAIR = Ed25519.parseKeyPair(
//   LOCAL_WALLET_PUBLIC,
//   LOCAL_WALLET_PRIVATE
// );

const DEPLOY_GAS_PRICE = 1;
export { DEPLOY_NODE_ADDRESS, DEPLOY_GAS_PAYMENT_FOR_SESSION_TRANSFER, MASTER_WALLET_KEYPAIR, DEPLOY_NODE_EVENT_STREAM_ADDRESS, DEPLOY_CHAIN_NAME, DEPLOY_GAS_PRICE, DEFAULT_TTL, DEPLOY_GAS_PAYMENT_FOR_INSTALL };