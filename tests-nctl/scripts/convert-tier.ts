import fs from "fs";
import path from 'path';
import { CLPublicKey } from "casper-js-sdk";
import testProject from "./tiers/test-project.json";
export const getAccountHashString = (publicKey: string) => {
  if (publicKey === "") return "";
  const accountHash = CLPublicKey.fromHex(publicKey).toAccountHashStr().slice(13);
  return accountHash;
};

const writeConvertedTier = async () => {
  const tiers = testProject.tier;
  const converted = testProject.casper.map(casperUser => {
    return {
      account: getAccountHashString(casperUser.publicKey),
      amount: tiers[casperUser.tier - 1]
    }
  });
  const output = {
    info: testProject.info,
    tiers: converted
  };
  fs.writeFileSync(path.resolve(__dirname, './tiers/test-project-casper.json'), JSON.stringify(output), 'utf8');
};

writeConvertedTier();