import { sleep } from "casper-js-client-helper/dist/helpers/utils";
import {
  CasperClient,
  Keys
} from "casper-js-sdk";
import * as fs from "fs";
const { Ed25519, AsymmetricKey } = Keys;
import _ from "lodash";
/**
 * Returns an on-chain account identifier.
 *
 * @param {AsymmetricKey} keyPair - Assymmetric keys of an on-chain account.
 * @return {String} Hexadecimal representation of an on-chain account identifier.
 */
export const getAccountHash = (keyPair: Keys.AsymmetricKey): string => {
  return Buffer.from(keyPair.accountHash()).toString("hex");
};

interface AccountInfo {
  namedKeys: any
}

/**
 * Returns on-chain account information.
 * @param {Object} client - JS SDK client for interacting with a node.
 * @param {String} stateRootHash - Root hash of global state at a recent block.
 * @param {Object} keyPair - Assymmetric keys of an on-chain account.
 * @return {Object} On-chain account information.
 */
export const getAccountInfo = async (client: CasperClient, stateRootHash: string, keyPair: Keys.AsymmetricKey): Promise<AccountInfo> => {
  const accountHash = getAccountHash(keyPair);
  const { Account: accountInfo } = await client.nodeClient.getBlockState(
    stateRootHash,
    `account-hash-${accountHash}`,
    []
  );

  return accountInfo!;
};

/**
 * Returns a value under an on-chain account's storage.
 * @param {CasperClient} client - JS SDK client for interacting with a node.
 * @param {String} stateRootHash - Root hash of global state at a recent block.
 * @param {Object} keyPair - Assymmetric keys of an on-chain account.
 * @param {String} namedKey - A named key associated with an on-chain account.
 * @return {String} On-chain account storage item value.
 */
export const getAccountNamedKeyValue = async (
  client: CasperClient,
  stateRootHash: string,
  keyPair: Keys.AsymmetricKey,
  namedKey: string
): Promise<string> => {
  // Chain query: get account information.
  const accountInfo = await getAccountInfo(client, stateRootHash, keyPair);
  // console.log("accountInfo:", accountInfo);
  // Get value of contract v1 named key.
  const { key: contractHash } = _.find(accountInfo.namedKeys, (i) => {
    return i.name === namedKey;
  });

  return contractHash;
};
/**
 * Returns global state root hash at current block.
 * @param client - JS SDK client for interacting with a node.
 * @return Root hash of global state at most recent block.
 */
export const getStateRootHash = async (client: CasperClient): Promise<string> => {
  const {
    block
  } = await client.nodeClient.getLatestBlockInfo();
  const {
    header: { state_root_hash: stateRootHash },
  } = block!;
  return stateRootHash;
};

/**
 * Returns a binary as u8 array.
 * @param {String} pathToBinary - Path to binary file to be loaded into memory.
 * @return {Uint8Array} Byte array.
 */
export const getBinary = (pathToBinary: string): Uint8Array => {
  return new Uint8Array(fs.readFileSync(pathToBinary, null).buffer);
};


export const getDeploy = async (NODE_URL: string, deployHash: string) => {
  const client = new CasperClient(NODE_URL);
  let i = 300;
  while (i != 0) {
    const [deploy, raw] = await client.getDeploy(deployHash);
    if (raw.execution_results.length !== 0) {
      // @ts-ignore
      if (raw.execution_results[0].result.Success) {
        return deploy;
      } else {
        // @ts-ignore
        throw Error(
          "Contract execution: " +
          // @ts-ignore
          raw.execution_results[0].result.Failure.error_message
        );
      }
    } else {
      i--;
      await sleep(1000);
      continue;
    }
  }
  throw Error("Timeout after " + i + "s. Something's wrong");
};


interface Argv {
  reamin: string[],
  cooked: string[],
  original: string[]
};
export const argv: Argv = JSON.parse(process.env.npm_config_argv || "");