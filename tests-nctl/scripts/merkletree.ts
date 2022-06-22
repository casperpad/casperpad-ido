import { encodeBase16, Keys } from "casper-js-sdk";
import { MerkleTree } from "merkletreejs";
import keccak256 from "keccak256";
import { BigNumberish, BigNumber } from "@ethersproject/bignumber";
import { assert } from "chai";
import kunft from "./tiers/casper-test/kunft.json";

type Root = Buffer;
type Leaf = Buffer;

type Position = "right" | "left";
interface Proof {
  position: Position;
  data: Buffer;
}

const verify = (root: Root, leaf: Leaf, proof: Proof[]) => {
  let computedHash = leaf;
  let i = 0;
  let temp: Buffer;
  for (i = 0; i < proof.length; i++) {
    const proofElement = proof[i];
    temp = computedHash;
    if (proofElement.position === "right") {
      // Hash(current computed hash + current element of the proof)
      computedHash = keccak256(
        Buffer.concat([computedHash, proofElement.data])
      );
    } else {
      // Hash(current element of the proof + current computed hash)
      computedHash = keccak256(
        Buffer.concat([proofElement.data, computedHash])
      );
    }
  }

  return computedHash.equals(root);
};

function logProof(proof: Proof) {
  console.log(proof.position, proof.data.toString("hex"));
}

function test_env_users(): string[] {
  const accountHashes = new Array(0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
    .reverse()
    .map((secret) => {
      const userSecret = new Uint8Array(new Array(32).fill(secret));
      const privateKey = Keys.Ed25519.parsePrivateKey(userSecret);
      const publicKey = Keys.Ed25519.privateToPublicKey(privateKey);
      const accountKey = Keys.Ed25519.parseKeyPair(publicKey, privateKey);
      return encodeBase16(accountKey.publicKey.toAccountHash());
    });
  return accountHashes;
}

type Tier = {
  account: string;
  amount: BigNumberish;
};

function get_tiers(): Tier[] {
  const test_users = test_env_users();

  return test_users.map((user, i) => {
    return {
      account: user,
      amount: BigNumber.from(i).mul(BigNumber.from(10).pow(18)).toString(),
    };
  });
}

function test_net_tiers(): Tier[] {
  return kunft.investors.map((investor) => {
    return {
      account: investor.accountHash,
      amount: kunft.tier[investor.tier],
    };
  });
}

export const genMerkleTree = () => {
  const tiers = test_net_tiers();
  const elements = tiers.map((tier) => `${tier.account}_${tier.amount}`);
  const leaves = elements.map(keccak256);
  const tree = new MerkleTree(leaves, keccak256);
  const root = tree.getRoot() as Root;
  const leaf = leaves[0];
  const proof = tree.getProof(leaf) as Proof[];

  console.log(tree.toString());

  let result = verify(root, leaf, proof);
  // assert(result);

  console.log(result);
  return { root, proof };
};

genMerkleTree();
