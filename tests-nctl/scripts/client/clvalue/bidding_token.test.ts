import { expect } from 'chai';
import { CLBiddingToken, CLBiddingTokenBytesParser, CLBiddingTokenType } from "./index";
import { BigNumber } from "@ethersproject/bignumber"

describe('CLBiddingToken', () => {
  it('Should be able to return proper value by calling .value()', () => {
    const biddingToken = { price: 2000 };
    const myHash = new CLBiddingToken(biddingToken);

    expect(myHash.value()).to.be.deep.eq(biddingToken);
  });

  it('Should be able to return proper value by calling .clType()', () => {
    const biddingToken = { price: 2000 };
    const myHash = new CLBiddingToken(biddingToken);

    expect(myHash.clType().toString()).to.be.eq('BiddingToken');
  });

  it('Should be able to return proper byte array by calling toBytes() / fromBytes()', () => {
    const expectedBytes = new Uint8Array([0, 1, 2, 208, 7]);
    const biddingToken = { price: BigNumber.from(2000) };
    const myHash = new CLBiddingToken(biddingToken);
    const bytes = new CLBiddingTokenBytesParser().toBytes(myHash).unwrap();

    expect(bytes).to.deep.eq(expectedBytes);
    expect(
      new CLBiddingTokenBytesParser().fromBytes(bytes, new CLBiddingTokenType()).unwrap()
    ).to.deep.eq(myHash);
  });

})