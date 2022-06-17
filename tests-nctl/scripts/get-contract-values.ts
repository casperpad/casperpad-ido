import { config } from "dotenv";
// config();
config({ path: '.env.development.local' });
import {
  CasperClient,
  CLAccountHash,
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



  const user = "account-hash-f2af240a5aa234d6e295ff65b011126dc002f655b1034f869f38b7b2ba60e450";

  const shedules = await idoContract.schedules();
  const payToken = await idoContract.payToken();
  // const claim = await idoContract.claimOf(user);
  const order = await idoContract.orderOf(user);

  console.dir({ payToken, order }, { depth: null });
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

const testERC20 = async () => {
  const casperClient = new CasperClient(NODE_ADDRESS!);
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
  try {
    const contractBalance = await erc20.balanceOf(CLValueBuilder.byteArray(decodeBase16(idoContractPackageHash.slice(5))));
    const accountBalance = await erc20.balanceOf(new CLAccountHash(decodeBase16("243598b8ac367f970dbc9b30c2dd866d85ab1a3902800adfb816eb3638d1bc1e")));
    console.log({ contractBalance, accountBalance });
  } catch (error: any) {
    console.error(error);
  }
}

test();

testERC20();

// testFactory();