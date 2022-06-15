import { config } from "dotenv";
// config();
config({ path: '.env.development.local' });
import {
  CasperClient,
  CLValueBuilder,
  decodeBase16,
  Keys,
} from "casper-js-sdk";
import { ERC20Client } from "casper-erc20-js-client";

import IDOClient from "./client/IDOClient";
import { getAccountInfo, getAccountNamedKeyValue, getDeploy } from "./utils";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MASTER_KEY_PAIR_PATH,
} = process.env;

const private_key = Keys.Ed25519.parsePrivateKeyFile(`${MASTER_KEY_PAIR_PATH}/secret_key.pem`);
const public_key = Keys.Ed25519.privateToPublicKey(private_key);

const KEYS = Keys.Ed25519.parseKeyPair(public_key, private_key);

const DEFAULT_RUN_ENTRYPOINT_PAYMENT = "50000000000";

const setAuctionToken = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);
  // let accountInfo = await getAccountInfo(casperClient, KEYS.publicKey);

  const idoContractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `casper_ido_contract_hash`
  );

  await idoContract.setContractHash(idoContractHash.slice(5));

  const erc20 = new ERC20Client(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );

  const erc20ContractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `Test Swappery Token_contract_hash`
  );

  await erc20.setContractHash(erc20ContractHash.slice(5));

  const idoContractPackageHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `casper_ido_contract_package_hash`
  );

  let deployHash = await erc20.approve(
    KEYS,
    CLValueBuilder.byteArray(decodeBase16(idoContractPackageHash.slice(5))),
    "1000000000000000",
    DEFAULT_RUN_ENTRYPOINT_PAYMENT
  );
  console.log(`ERC20 Approve deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("Approved to contract.");

  deployHash = await idoContract.setAuctionToken(
    KEYS,
    `contract-${erc20ContractHash.slice(5)}`,
    DEFAULT_RUN_ENTRYPOINT_PAYMENT
  );

  console.log(`setAuctionToken deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("setAuctionToken done");

};

const setMerkelRoot = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);

  const idoContractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `casper_ido_contract_hash`
  );

  await idoContract.setContractHash(idoContractHash.slice(5));

  const deployHash = await idoContract.setMerkleRoot(
    KEYS,
    "95ec4ac2a65d8711d48ad9fd42f3b157dc580b187608571fac48598fa108199e",
    DEFAULT_RUN_ENTRYPOINT_PAYMENT
  );
  console.log(`setMerkleRoot deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("setMerkleRoot done");

  console.log(`... Run successfully.`);
};

const runPresaleActions = async () => {
  await setAuctionToken();
  await setMerkelRoot();
}

setMerkelRoot();