import { config } from "dotenv";
config();
// config({ path: ".env.test.local" });
// config({ path: ".env.production.local" });
import {
  CasperClient,
  CLValueBuilder,
  decodeBase16,
  Keys,
} from "casper-js-sdk";
import { parseFixed } from "@ethersproject/bignumber";
import { ERC20Client } from "casper-erc20-js-client";
import { MerkleTree } from "merkletreejs";
import keccak256 from "keccak256";

import kunft from "./tiers/casper/kunft.json";

import IDOClient from "./client/IDOClient";
import { getAccountNamedKeyValue, getDeploy } from "./utils";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MASTER_KEY_PAIR_PATH,
  DEFAULT_RUN_ENTRYPOINT_PAYMENT,
} = process.env;

const private_key = Keys.Ed25519.parsePrivateKeyFile(
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);
const public_key = Keys.Ed25519.privateToPublicKey(private_key);

const KEYS = Keys.Ed25519.parseKeyPair(public_key, private_key);

function test_net_tiers() {
  return kunft.investors.map((investor) => {
    return {
      account: investor.accountHash,
      amount: kunft.tier[investor.tier],
    };
  });
}

export const genMerkleTree = () => {
  const tiers = test_net_tiers();
  const elements = tiers.map((tier) => `${tier.account}_${tier.amount}`);
  const leaves = elements.map(keccak256);
  const tree = new MerkleTree(leaves, keccak256);
  const root = tree.getHexRoot();
  return root;
};

const setAuctionToken = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );

  const idoContractHash =
    "hash-6cc90635ccaf0fac5c08f0533ea85336f2975202f5d01a5a1894f6f6cb306827";

  await idoContract.setContractHash(idoContractHash.slice(5));

  const erc20 = new ERC20Client(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );

  const erc20ContractHash =
    "hash-a3834fc5ebb9ae5e83bc3cf3b94a27728150bb4a19946fa64f4f663113d5f882";

  await erc20.setContractHash(erc20ContractHash.slice(5));

  const idoContractPackageHash =
    "hash-2d9d5859197fe59f125762634f5aa7a93bdd50c1cfeebf3276313da418a57d57";
  const auctionTokenCapacity = "3095087649937267";

  let deployHash = await erc20.approve(
    KEYS,
    CLValueBuilder.byteArray(decodeBase16(idoContractPackageHash.slice(5))),
    auctionTokenCapacity.toString(),
    "100000000"
  );
  console.log(`ERC20 Approve deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("Approved to contract.");

  deployHash = await idoContract.setAuctionToken(
    KEYS,
    `contract-${erc20ContractHash.slice(5)}`,
    auctionTokenCapacity,
    "1000000000"
  );

  console.log(`setAuctionToken deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("setAuctionToken done");
};

const changeAuctionTokenPrice = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );

  const idoContractHash =
    "hash-f86ccc2fb216ac512ddb44c42208cb86b7f75bf260f4505246def86d014a3c11";

  await idoContract.setContractHash(idoContractHash.slice(5));

  const deployHash = await idoContract.changeAuctionTokenPrice(
    KEYS,
    "1000000000",
    "1000000000"
  );

  console.log(`changeAuctionTokenPrice deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("changeAuctionTokenPrice done");
};

const setMerkelRoot = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);
  const idoContractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `KUNFT Marketplace_ido_contract_hash`
  );

  await idoContract.setContractHash(idoContractHash.slice(5));

  const root = genMerkleTree();

  const deployHash = await idoContract.setMerkleRoot(
    KEYS,
    root.slice(2),
    DEFAULT_RUN_ENTRYPOINT_PAYMENT!
  );
  console.log(`setMerkleRoot deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("setMerkleRoot done");

  console.log(`... Run successfully.`);
};

const runPresaleActions = async () => {
  await setAuctionToken();
  await setMerkelRoot();
};

setAuctionToken();
// changeAuctionTokenPrice();
// runPresaleActions();
// setMerkelRoot();
