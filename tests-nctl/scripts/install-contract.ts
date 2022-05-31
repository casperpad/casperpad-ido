/**
 * @fileOverview CSPR JS SDK demo: ERC20 - install contract.
 */

import {
  CasperClient,
  CLValueBuilder,
  DeployUtil,
  RuntimeArgs,
} from "casper-js-sdk";
import * as constants from "./constants";
import * as utils from "./utils";

// Path to contract to be installed.
const PATH_TO_CONTRACT = "/home/master/workspace/casperpad-ido/tests/wasm/casper_ido_contract.wasm";

/**
 * Demonstration entry point.
 */
const main = async () => {
  // Step 1: Set casper node client.
  const client = new CasperClient(constants.DEPLOY_NODE_ADDRESS);
  // Step 2: Set contract installation deploy (unsigned).
  let deploy = DeployUtil.makeDeploy(
    new DeployUtil.DeployParams(
      constants.MASTER_WALLET_KEYPAIR.publicKey,
      constants.DEPLOY_CHAIN_NAME,
    ),
    DeployUtil.ExecutableDeployItem.newModuleBytes(
      utils.getBinary(PATH_TO_CONTRACT),
      RuntimeArgs.fromMap({
        "default_treasury_wallet": CLValueBuilder.key(CLValueBuilder.byteArray(constants.MASTER_WALLET_KEYPAIR.accountHash())),
        "contract_name": CLValueBuilder.string("casper_ido")
      })
    ),
    DeployUtil.standardPayment(constants.DEPLOY_GAS_PAYMENT_FOR_INSTALL)
  );

  // Step 3: Sign deploy.
  deploy = client.signDeploy(deploy, constants.MASTER_WALLET_KEYPAIR);

  // Step 4: Dispatch deploy to node.
  const deployhash = await client.putDeploy(deploy);

  // Step 5: Render deploy details.
  console.log({ deployhash });
  const result = await utils.getDeploy(constants.DEPLOY_NODE_ADDRESS, deployhash);
  console.log(result);
};



main();