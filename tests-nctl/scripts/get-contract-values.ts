import { config } from "dotenv";
config();
// config({ path: ".env.test.local" });
import {
  CasperClient,
  CLAccountHash,
  CLValueBuilder,
  decodeBase16,
  Keys,
} from "casper-js-sdk";
import { ERC20Client } from "casper-erc20-js-client";
import kunft from "./tiers/casper/kunft.json";

import IDOClient from "./client/IDOClient";
import { getAccountNamedKeyValue } from "./utils";
import FactoryClient from "./client/FactoryClient";

const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, MASTER_KEY_PAIR_PATH } =
  process.env;

const KEYS = Keys.Ed25519.loadKeyPairFromPrivateFile(
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);

const test = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);
  const idoContractHash =
    "hash-74764b918573d0572fe949c434c295204d4d4afa70e067b9bb21082306da8e87";

  await idoContract.setContractHash(idoContractHash.slice(5));

  const user =
    "account-hash-f2af240a5aa234d6e295ff65b011126dc002f655b1034f869f38b7b2ba60e450";

  const shedules = await idoContract.schedules();
  const payToken = await idoContract.payToken();

  console.dir({ shedules }, { depth: null });
};

const testERC20 = async () => {
  const casperClient = new CasperClient(NODE_ADDRESS!);
  const erc20 = new ERC20Client(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const erc20ContractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `Test Swappery Token_contract_hash`
  );

  await erc20.setContractHash(erc20ContractHash.slice(5));

  const idoContractPackageHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `casper_ido_contract_package_hash`
  );
  try {
    const contractBalance = await erc20.balanceOf(
      CLValueBuilder.byteArray(decodeBase16(idoContractPackageHash.slice(5)))
    );
    const accountBalance = await erc20.balanceOf(
      new CLAccountHash(
        decodeBase16(
          "243598b8ac367f970dbc9b30c2dd866d85ab1a3902800adfb816eb3638d1bc1e"
        )
      )
    );
    console.log({ contractBalance, accountBalance });
  } catch (error: any) {
    console.error(error);
  }
};

const fetchOrdersOfWhitelistedUsers = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);
  const idoContractHash =
    "hash-74764b918573d0572fe949c434c295204d4d4afa70e067b9bb21082306da8e87";

  await idoContract.setContractHash(idoContractHash.slice(5));

  const user =
    "account-hash-f2af240a5aa234d6e295ff65b011126dc002f655b1034f869f38b7b2ba60e450";

  const promises = kunft.investors.map(async (hash) => {
    const amount = await idoContract.orderOf(`${hash.accountHash}`);
    if (amount)
      console.dir({ accountHash: hash.accountHash, amount }, { depth: null });
  });
  await Promise.all(promises);
};

fetchOrdersOfWhitelistedUsers();
