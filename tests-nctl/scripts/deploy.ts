import {
  CasperClient
} from "casper-js-sdk";
import * as utils from "./utils";
import * as constants from "./constants";
const main = async () => {
  const client = new CasperClient(constants.DEPLOY_NODE_ADDRESS);
  const deploy_hash = utils.argv.original[1];
  const deploy_info = await client.nodeClient.getDeployInfo(deploy_hash);
  console.dir(deploy_info, { depth: null });
}
main();