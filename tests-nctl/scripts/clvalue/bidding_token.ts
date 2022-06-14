import { CLErrorCodes, CLMap, CLMapBytesParser, CLMapType, CLOption, CLOptionBytesParser, CLOptionType, CLString, CLStringType, CLType, CLTypeTag, CLU256, CLU256Type, CLValue, CLValueBuilder, CLValueBytesParsers, ResultAndRemainder, resultHelper, ToBytesResult } from "casper-js-sdk";
import { BigNumberish } from "@ethersproject/bignumber";
import { Ok, Err, Some, None } from 'ts-results';

enum BiddingTokenVariant {
  Native,
  ERC20s
}
/**
 * User cusotimzed CLType
 * @deprecated
 */
export class CLBiddingTokenType extends CLType {
  tag = CLTypeTag.Any;
  linksTo = CLBiddingToken;

  toString(): string {
    return "BiddingToken";
  }

  toJSON(): string {
    return this.toString();
  }

}

export type Native = { price?: BigNumberish };

export type ERC20s = { tokens_with_price: Map<string, BigNumberish> }

export type BiddingToken = Native | ERC20s;

function isERC20s(biddingToken: BiddingToken): biddingToken is ERC20s {
  return (biddingToken as ERC20s).tokens_with_price !== undefined;
}

function concatUint8Array(a: Uint8Array, b: Uint8Array) { // a, b TypedArray of same type
  const c = new Uint8Array(a.length + b.length);
  c.set(a, 0);
  c.set(b, a.length);
  return c;
}

export class CLBiddingTokenBytesParser extends CLValueBytesParsers {
  toBytes(value: CLBiddingToken): ToBytesResult {

    const { data } = value;

    if (isERC20s(data)) {

      const converted = Array.from(data.tokens_with_price.entries()).map((a) => {
        return [CLValueBuilder.string(a[0]), CLValueBuilder.u256(a[1])] as [CLString, CLU256];
      });
      const clMap = new CLMap<CLString, CLU256>(converted);
      const bytes = new CLMapBytesParser().toBytes(clMap).unwrap();

      const result = concatUint8Array(new Uint8Array(BiddingTokenVariant.Native), bytes);
      return Ok(new Uint8Array(result));
    } else {

      const myType = new CLOptionType(new CLU256Type());
      let optionPrice: CLOption<CLValue> = new CLOption(None, new CLU256Type());
      if (data.price)
        optionPrice = new CLOption(Some(new CLU256(data.price)));

      const bytes = new CLOptionBytesParser().toBytes(optionPrice).unwrap();

      const result = concatUint8Array(new Uint8Array(BiddingTokenVariant.ERC20s), bytes);
      return Ok(new Uint8Array(result));
    }
  }


  fromBytesWithRemainder(
    bytes: Uint8Array
  ): ResultAndRemainder<CLBiddingToken, CLErrorCodes> {
    if (bytes.length < 1) {
      return resultHelper(Err(CLErrorCodes.EarlyEndOfStream));
    }
    const tag = bytes[0];

    if (tag === BiddingTokenVariant.Native) {
      const myType = new CLOptionType(new CLU256Type());
      const {
        result: priceResult,
        remainder
      } = new CLOptionBytesParser().fromBytesWithRemainder(bytes.subarray(1), myType);
      if (priceResult.ok) {
        let native: Native = { price: undefined };
        if (priceResult.val.isSome()) {
          native = { price: (priceResult.val.value().unwrap() as CLU256).data }
        }
        return resultHelper(Ok(new CLBiddingToken(native)), remainder);
      } else {
        return resultHelper(Err(priceResult.val));
      }
    } else if (bytes[0] === BiddingTokenVariant.ERC20s) {
      const mapType = new CLMapType([new CLStringType(), new CLU256Type()]);
      const { result: mapResult,
        remainder
      } = new CLMapBytesParser().fromBytesWithRemainder(bytes.subarray(1), mapType);
      if (mapResult.ok) {
        const result = new Map(mapResult.val.data.map(a => { return [(a[0] as CLString).data, (a[1] as CLU256).data] as [string, BigNumberish] }));
        const erc20s: ERC20s = { tokens_with_price: result };
        return resultHelper(Ok(new CLBiddingToken(erc20s)), remainder);
      } else {
        return resultHelper(Err(mapResult.val));
      }
    } else {
      return resultHelper(Err(CLErrorCodes.Formatting));
    }
  }
}

export class CLBiddingToken extends CLValue {
  data: BiddingToken;
  bytesParser: CLBiddingTokenBytesParser;

  constructor(v: BiddingToken) {
    super();
    this.bytesParser = new CLBiddingTokenBytesParser();
    this.data = v;
  }

  clType(): CLType {
    return new CLBiddingTokenType();
  }

  value(): BiddingToken {
    return this.data;
  }
}