import { CLKey, CLValue, CLValueParsers } from "casper-js-sdk";
import { concat } from "@ethersproject/bytes";
import blake from "blakejs";


export const keyAndValueToHex = (key: CLKey, value: CLValue) => {
  const aBytes = CLValueParsers.toBytes(key).unwrap();
  const bBytes = CLValueParsers.toBytes(value).unwrap();

  const blaked = blake.blake2b(concat([aBytes, bBytes]), undefined, 32);
  const hex = Buffer.from(blaked).toString('hex');

  return hex;
}
