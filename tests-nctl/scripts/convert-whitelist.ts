import fs from "fs";
import path from "path";
import { CLPublicKey } from "casper-js-sdk";
import whitelist from "./tiers/whitelist/test.json";

export const getAccountHashString = (publicKey: string) => {
  if (publicKey === "") return "";
  const accountHash = CLPublicKey.fromHex(publicKey)
    .toAccountHashStr()
    .slice(13);
  return accountHash;
};

const writeConvertedTier = async () => {
  const converted = whitelist.investors.map((casperUser) => {
    return {
      accountHash: getAccountHashString(casperUser.publicKey),
      tier: casperUser.tier,
    };
  });
  const output = {
    investors: converted,
  };
  fs.writeFileSync(
    path.resolve(__dirname, "./tiers/whitelist/converted.json"),
    JSON.stringify(output),
    "utf8"
  );
};

writeConvertedTier();
