import {
  CLMap,
  CLStringType,
  CLU256,
  CLU64,
  CLValueBuilder,
  Keys,
  RuntimeArgs,
} from "casper-js-sdk";
import { CasperContractClient, helpers, constants } from "casper-js-client-helper";
import { BigNumberish } from '@ethersproject/bignumber';
import { Ok, Err, Some, None } from 'ts-results';
import { BiddingToken, CLBiddingToken, CLBiddingTokenBytesParser } from "../clvalue";

const {
  fromCLMap,
  toCLMap,
  installContract,
  setClient,
  contractSimpleGetter,
  contractCallFn,
  createRecipientAddress
} = helpers;

const { DEFAULT_TTL } = constants;

export default class IDOClient extends CasperContractClient {
  protected namedKeys?: {
    allowances: string;
    balances: string;
  };

  /**
   * Installs the ERC20 contract.
   *
   * @param keys AsymmetricKey that will be used to install the contract.
   * @param contractName Name of the Factory contract.
   * @param treasuryWallet treasury wallet.
   * @param feeDenominator Specifies fee denominator.
   * @param paymentAmount The payment amount that will be used to install the contract.
   * @param wasmPath Path to the WASM file that will be installed.
   *
   * @returns Installation deploy hash. 
   */
  public async install(
    keys: Keys.AsymmetricKey,
    contractName: string,
    factoryContract: string,
    info: string,
    auctionStartTime: number,
    auctionEndTime: number,
    launchTime: number,
    auctionTokenPrice: BigNumberish,
    auctionTokenCapacity: BigNumberish,
    biddingToken: BiddingToken,
    schedules: Map<number, BigNumberish>,
    paymentAmount: string,
    wasmPath: string,
    auctionToken?: string,
  ) {
    if (schedules.size === 0) {
      throw Error("Map size muste be greater than zero");
    }
    const converted = Array.from(schedules.entries()).map((a) => {
      return [CLValueBuilder.u64(a[0]), CLValueBuilder.u256(a[1])] as [CLU64, CLU256];
    });
    const clMap = new CLMap<CLU64, CLU256>(converted);
    const runtimeArgs = RuntimeArgs.fromMap({
      contract_name: CLValueBuilder.string(contractName),
      factory_contract: CLValueBuilder.string(factoryContract),
      info: CLValueBuilder.string(info),
      auction_start_time: CLValueBuilder.u64(auctionStartTime),
      auction_end_time: CLValueBuilder.u64(auctionEndTime),
      launch_time: CLValueBuilder.u64(launchTime),
      auction_token_price: CLValueBuilder.u256(auctionTokenPrice),
      auction_token_capacity: CLValueBuilder.u256(auctionTokenCapacity),
      bidding_token: CLValueBuilder.byteArray(new CLBiddingTokenBytesParser().toBytes(new CLBiddingToken(biddingToken)).unwrap()),
      auction_token: auctionToken ? CLValueBuilder.option(Some(CLValueBuilder.string(auctionToken))) : CLValueBuilder.option(None, new CLStringType()),
      schedules: clMap,
    });

    return await installContract(
      this.chainName,
      this.nodeAddress,
      keys,
      runtimeArgs,
      paymentAmount,
      wasmPath
    );
  }

  /**
   * Set ERC20 contract hash so its possible to communicate with it.
   *
   * @param hash Contract hash (raw hex string as well as `hash-` prefixed format is supported).
   */
  public async setContractHash(hash: string) {
    const properHash = hash.startsWith("hash-") ? hash.slice(5) : hash;
    const { contractPackageHash, namedKeys } = await setClient(
      this.nodeAddress,
      properHash,
      [
        "balances",
        "allowances"
      ]
    );
    this.contractHash = hash;
    this.contractPackageHash = contractPackageHash;
    /* @ts-ignore */
    this.namedKeys = namedKeys;
  }

  public async setTreasuryWallet(
    keys: Keys.AsymmetricKey,
    treasuryWallet: string,
    paymentAmount: string,
    ttl = DEFAULT_TTL
  ) {
    const runtimeArgs = RuntimeArgs.fromMap({
      treasury_wallet: CLValueBuilder.string(treasuryWallet),
    });

    return await this.contractCall({
      entryPoint: "transfer",
      keys,
      paymentAmount,
      runtimeArgs,
      ttl,
    });
  }

  /**
   * Returns the treasuryWallet. 
   */
  public async treasuryWallet() {
    return await contractSimpleGetter(
      this.nodeAddress,
      this.contractHash!,
      ["treasury_wallet"]
    );
  }


  /**
   * Returns the feeDenominator. 
   */
  public async feeDenominator() {
    return await contractSimpleGetter(
      this.nodeAddress,
      this.contractHash!,
      ["fee_denominator"]
    );
  }
}