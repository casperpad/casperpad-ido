import {
  CLMap,
  CLStringType,
  CLU256,
  CLU64,
  CLValueBuilder,
  Keys,
  RuntimeArgs, decodeBase16, CLAccountHash, CLString, CLU256Type
} from "casper-js-sdk";
import { CasperContractClient, helpers, constants, utils } from "casper-js-client-helper";
import { BigNumberish } from '@ethersproject/bignumber';
import { Ok, Err, Some, None } from 'ts-results';
import { keyAndValueToHex } from "./utlis";

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
    auction_end_time: string;
    auction_start_time: string;
    auction_token_capacity: string;
    auction_token_price: string;
    bidding_token: string;
    claims: string;
    creator: string;
    factory_contract: string;
    info: string;
    launch_time: string;
    merkle_root: string;
    orders: string;
    reentrancy_guard: string;
    schedules: string;
    total_participants: string;
    sold_amount: string;
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
    schedules: Map<number, BigNumberish>,
    paymentAmount: string,
    wasmPath: string,
    payToken?: string,
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
      pay_token: payToken ? CLValueBuilder.option(Some(CLValueBuilder.string(payToken))) : CLValueBuilder.option(None, new CLStringType()),
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
   * Set contract hash so its possible to communicate with it.
   *
   * @param hash Contract hash (raw hex string as well as `hash-` prefixed format is supported).
   */
  public async setContractHash(hash: string) {
    const properHash = hash.startsWith("hash-") ? hash.slice(5) : hash;
    const { contractPackageHash, namedKeys } = await setClient(
      this.nodeAddress,
      properHash,
      [
        "auction_end_time",
        "auction_start_time",
        "auction_token_capacity",
        "auction_token_price",
        "pay_token",
        "claims",
        "creator",
        "factory_contract",
        "info",
        "launch_time",
        "merkle_root",
        "orders",
        "reentrancy_guard",
        "schedules",
        "total_participants",
        "sold_amount"
      ]
    );
    this.contractHash = hash;
    this.contractPackageHash = contractPackageHash;
    /* @ts-ignore */
    this.namedKeys = namedKeys;
  }

  public async queryContract(key: string) {
    return await contractSimpleGetter(this.nodeAddress, this.contractHash!, [key])
  }

  async queryContractDictionary(dictionary: string, key: string) {

    const result = await utils.contractDictionaryGetter(
      this.nodeAddress,
      key,
      /* @ts-ignore */
      this.namedKeys![dictionary]
    );
    return result;
  }

  public async result() {
    return await contractSimpleGetter(
      this.nodeAddress,
      this.contractHash!,
      ["result"]
    );
  }

  public async schedules(): Promise<[BigNumberish, BigNumberish][]> {
    const result: [CLU64, CLU256][] = await this.queryContract("schedules");
    return result.map(schedule => {
      return [schedule[0].data.toNumber(), schedule[1].data.toNumber()] as [BigNumberish, BigNumberish]
    }
    );
  }

  public async payToken(): Promise<Some<CLString>> {
    return await this.queryContract("pay_token");
  }

  public async claimOf(account: string, time: number) {
    const clAccount = CLValueBuilder.key(new CLAccountHash(decodeBase16(account)));
    const clTime = CLValueBuilder.u64(time);
    const key = keyAndValueToHex(clAccount, clTime);
    try {
      return await this.queryContractDictionary("claims", key);
    } catch (error: any) {
      return undefined;
    }
  }

  public async orderOf(account: string): Promise<Some<CLU256> | undefined> {
    try {
      const preferKey = account.startsWith("account-hash-") ? account.slice(13) : account;
      return await this.queryContractDictionary("orders", preferKey);
    } catch (error: any) {
      return undefined;
    }
  }

  public async setMerkleRoot(
    keys: Keys.AsymmetricKey,
    merkleRoot: string,
    paymentAmount: string,
    ttl = DEFAULT_TTL,) {
    const runtimeArgs = RuntimeArgs.fromMap({
      merkle_root: CLValueBuilder.string(merkleRoot),
    });

    return await this.contractCall({
      entryPoint: "set_merkle_root",
      keys,
      paymentAmount,
      runtimeArgs,
      ttl,
    });
  }

  public async setAuctionToken(
    keys: Keys.AsymmetricKey,
    auctionToken: string,
    paymentAmount: string,
    ttl = DEFAULT_TTL,) {
    const runtimeArgs = RuntimeArgs.fromMap({
      auction_token: CLValueBuilder.string(auctionToken),
    });

    return await this.contractCall({
      entryPoint: "set_auction_token",
      keys,
      paymentAmount,
      runtimeArgs,
      ttl,
    });
  }

  public async addOrders(
    keys: Keys.AsymmetricKey,
    orders: Map<string, BigNumberish>,
    paymentAmount: string,
    ttl = DEFAULT_TTL,) {
    const clOrders = new CLMap([new CLStringType(), new CLU256Type()]);
    orders.forEach((orderAmout, account) => {
      clOrders.set(CLValueBuilder.string(account), CLValueBuilder.u256(orderAmout));
    });
    const runtimeArgs = RuntimeArgs.fromMap({
      orders: clOrders,
    });

    return await this.contractCall({
      entryPoint: "add_orders",
      keys,
      paymentAmount,
      runtimeArgs,
      ttl,
    });
  }
}