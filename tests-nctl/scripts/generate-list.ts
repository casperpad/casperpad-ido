import fs from "fs";
import path from "path";
import { CLPublicKey } from "casper-js-sdk";
import { BigNumber } from "@ethersproject/bignumber";
import { investors as bscInvestors } from "./tiers/casper/bsc.json";
import { investors as casperInvestors } from "./tiers/casper/casper.json";

function generateList() {
  const price = "2612999994";
  let investors = bscInvestors.map((investor) => {
    // investor amount is given in usdamount*100 since 1knft = 0.01usd
    return {
      accountHash: CLPublicKey.fromHex(investor.publicKey)
        .toAccountHashStr()
        .slice("account-hash-".length),
      amount: BigNumber.from(investor.amount)
        .mul(10 ** 9)
        .toString(),
    };
  });
  investors = investors.concat(
    casperInvestors.map((investor) => {
      return {
        accountHash: investor.accountHash,
        amount: BigNumber.from(investor.amount)
          .mul(price)
          .div(10 ** 9)
          .toString(),
      };
    })
  );

  let auctionTokenAmount = BigNumber.from(0);
  investors.forEach((i) => {
    auctionTokenAmount = auctionTokenAmount.add(i.amount);
  });

  console.log(auctionTokenAmount.toString());

  fs.writeFileSync(
    path.resolve(__dirname, "./tiers/casper/converted.json"),
    JSON.stringify({ investors }),
    "utf8"
  );
}

generateList();
