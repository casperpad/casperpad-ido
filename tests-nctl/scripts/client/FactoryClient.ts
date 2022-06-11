import {
  CLValueBuilder,
  Keys,
  RuntimeArgs,
} from "casper-js-sdk";
import { CasperContractClient, helpers, constants } from "casper-js-client-helper";

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

export default class FactoryClient extends CasperContractClient {
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
    treasuryWallet: string,
    feeDenominator: number,
    paymentAmount: string,
    wasmPath: string
  ) {
    const runtimeArgs = RuntimeArgs.fromMap({
      contract_name: CLValueBuilder.string(contractName),
      treasury_wallet: CLValueBuilder.string(treasuryWallet),
      fee_denominator: CLValueBuilder.u256(feeDenominator),
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