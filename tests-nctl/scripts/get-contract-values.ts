import { config } from "dotenv";
config();
// config({ path: '.env.development.local' });
import {
  CasperClient,
  CLValueBuilder,
  decodeBase16,
  Keys,
} from "casper-js-sdk";
import { ERC20Client } from "casper-erc20-js-client";

import IDOClient from "./client/IDOClient";
import { getAccountNamedKeyValue } from "./utils";
import FactoryClient from "./client/FactoryClient";


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

const test = async () => {
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
  const biddingToken = await idoContract.biddingToken();
  console.log({ biddingToken });
};

const testFactory = async () => {
  const factoryContract = new FactoryClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);
  const factoryContractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `ido_factory_contract_hash`
  );
  await factoryContract.setContractHash(factoryContractHash.slice(5));

  const installTime = await factoryContract.installTime();

  console.log(installTime.toString());
}

test();

// testFactory();