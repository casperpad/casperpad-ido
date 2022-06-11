import {
  encodeBase16,
  Keys
} from "casper-js-sdk";

import { MerkleTree } from "merkletreejs";
import keccak256 from 'keccak256';
import { concat } from "@ethersproject/bytes";

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
  const accountHashes = new Array(0, 1, 2, 3, 4, 5, 6, 7, 8, 9).map(secret => {
    const userSecret = new Uint8Array(new Array(32).fill(secret));
    const privateKey = Keys.Ed25519.parsePrivateKey(userSecret);
    const publicKey = Keys.Ed25519.privateToPublicKey(privateKey);
    console.log(encodeBase16(publicKey))
    const accountKey = Keys.Ed25519.parseKeyPair(publicKey, privateKey);
    return encodeBase16(accountKey.publicKey.toAccountHash());
  })
  return accountHashes;
}

export const genMerkleTree = () => {
  const leaves = [
    "f4668ecfd97223a9e9087b573eef139cb0b5adc513dfb3e8c248e802c571ed4b",
    "105b69f2d74a211a6cb337cba6751a8f15cc7b44b7c65329c29731b67e1ac047",
    "3d5de8c609159a0954e773dd686fb7724428316cb30e00bdc899976127747f55",
    "2f492f5e3cf699b89a9b68293b8ac258f9e533d1212dda478ed98844b462f966",
  ].map(keccak256);
  const tree = new MerkleTree(leaves, keccak256);
  const root = tree.getRoot() as Root;
  const leaf = keccak256('f4668ecfd97223a9e9087b573eef139cb0b5adc513dfb3e8c248e802c571ed4b') as Leaf;
  const proof = tree.getProof(leaf) as Proof[];

  const res = tree.verify(proof, leaf, root);



  proof.forEach(logProof);

  console.log("----");
  console.log(tree.toString());
  console.log("----");

  const result = verify(root, leaf, proof);


  return { root, proof };

}

// genMerkleTree();

console.log(test_env_users());