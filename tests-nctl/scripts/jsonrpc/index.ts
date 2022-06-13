import {
  CasperServiceByJsonRPC, CLJSONFormat,
} from "casper-js-sdk";

export class CustomizedCasperServiceByJsonRPC extends CasperServiceByJsonRPC {
  public async getBlockState1(
    stateRootHash: string,
    key: string,
    path: string[]
  ): Promise<CLJSONFormat> {
    const res = await this.client.request({
      method: 'state_get_item',
      params: {
        state_root_hash: stateRootHash,
        key,
        path
      }
    });
    if (res.error) {
      return res;
    } else {
      const storedValueJson = res.stored_value;
      return storedValueJson['CLValue'];
    }
  }
}