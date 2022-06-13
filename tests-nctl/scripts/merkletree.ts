import {
  encodeBase16,
  Keys
} from "casper-js-sdk";

import { MerkleTree } from "merkletreejs";
import keccak256 from 'keccak256';
import { BigNumberish, BigNumber } from "@ethersproject/bignumber";
import { assert } from "chai";

type Root = Buffer;
type Leaf = Buffer;

type Position = "right" | "left";
interface Proof {
  position: Position,
  data: Buffer,
}

const verify = (root: Root, leaf: Leaf, proof: Proof[]) => {
  let computedHash = leaf;
  let i = 0;
  let temp: Buffer;
  for (i = 0; i < proof.length; i++) {
    const proofElement = proof[i];
    temp = computedHash;
    if (proofElement.position === 'right') {
      // Hash(current computed hash + current element of the proof)
      computedHash = keccak256(Buffer.concat([computedHash, proofElement.data]));
    } else {
      // Hash(current element of the proof + current computed hash)
      computedHash = keccak256(Buffer.concat([proofElement.data, computedHash]));
    }
  }

  return computedHash.equals(root);

}

function logProof(proof: Proof) {
  console.log(proof.position, proof.data.toString('hex'));
}

function test_env_users(): string[] {
  const accountHashes = new Array(0, 1, 2, 3, 4, 5, 6, 7, 8, 9).reverse().map(secret => {
    const userSecret = new Uint8Array(new Array(32).fill(secret));
    const privateKey = Keys.Ed25519.parsePrivateKey(userSecret);
    const publicKey = Keys.Ed25519.privateToPublicKey(privateKey);
    const accountKey = Keys.Ed25519.parseKeyPair(publicKey, privateKey);
    return encodeBase16(accountKey.publicKey.toAccountHash());
  })
  return accountHashes;
}

type Tier = {
  account: string;
  amount: BigNumberish;
}

function get_tiers(): Tier[] {
  const test_users = test_env_users();

  return test_users.map((user, i) => {
    return {
      account: user,
      amount: BigNumber.from(i).mul(BigNumber.from(10).pow(18)).toString()
    }
  })
}

function test_net_tiers(): Tier[] {
  return [
    {
      account: '2642243a3ca1abc6f1b5ad3c9f53114955533ffe1a9e76055d1f987370d1d8e0',
      amount: '500000000000'
    },
    {
      account: '243598b8ac367f970dbc9b30c2dd866d85ab1a3902800adfb816eb3638d1bc1e',
      amount: '100000000000'
    },
    {
      account: 'e4b0574a11b11e0d7adc6529fb0e4fc3d1db04cccc9883e171273ddf6e095762',
      amount: '200000000000'
    },
    {
      account: '17e640bf6133115e130c196ff7e29017216680fd1bd506bdb8664d842b7c9a5c',
      amount: '50000000000'
    },
    {
      account: 'bf121f7a912df3dfe24de5b7c1c0e9e7cb1c7a910854fe227a2dc35fdb7b18b5',
      amount: '500000000000'
    },
    {
      account: '243598b8ac367f970dbc9b30c2dd866d85ab1a3902800adfb816eb3638d1bc1e',
      amount: '100000000000'
    }
  ]
}

export const genMerkleTree = () => {
  // const tiers = get_tiers();
  const tiers = test_net_tiers();
  const elements = tiers.map(tier => `${tier.account}_${tier.amount}`);
  const leaves = elements.map(keccak256);
  const tree = new MerkleTree(leaves, keccak256);
  const root = tree.getRoot() as Root;
  const leaf = leaves[2];
  const proof = tree.getProof(leaf) as Proof[];

  proof.forEach(logProof);

  console.log("----");
  console.log(tree.toString());
  console.log("----");

  const result = verify(root, leaf, proof);
  assert(result);

  return { root, proof };

}

genMerkleTree();