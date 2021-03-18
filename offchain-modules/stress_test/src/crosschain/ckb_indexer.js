const {
    Collector,
    Amount,
    Cell,
    Script,
    OutPoint,
    AmountUnit,
} = require("@lay2/pw-core");
const {CKB_INDEXER_URL} = require("./config");
const axios = require('axios')
const {sleep} = require("./method");

class SDCollector extends Collector {
    indexerUrl = CKB_INDEXER_URL;

    constructor() {
        super();
    }

    async getCells(address) {
        this.cells = [];
        let postData = {
            "id": 2,
            "jsonrpc": "2.0",
            "method": "get_cells",
            "params": [
                {
                    "script": {
                        "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
                        "hash_type": "type",
                        "args": address
                    },
                    "script_type": "lock"
                },
                "asc",
                "0x64"
            ]
        };
        let res;
        while (res === "" || res === undefined || res == null) {
            try {
                res = await axios.post(`${this.indexerUrl}`, postData);
                console.log("indexer response", res.data)
            }catch (error) {
                console.error("failed to get indexer data", error)
            }
            await sleep(5*1000)
        }
       
        const rawCells = res.data.result.objects;
        console.log("inderer post response", rawCells )
        for (let rawCell of rawCells) {
            // const cell = new Cell(
            //     new Amount(rawCell.output.capacity, AmountUnit.shannon),
            //     Script.fromRPC(rawCell.output.lock),
            //     Script.fromRPC(rawCell.output.type),
            //     OutPoint.fromRPC(rawCell.out_point),
            //     rawCell.output_data
            // );
            const cell = {
                capacity: rawCell.output.capacity,
                lock: Script.fromRPC(rawCell.output.lock),
                type: Script.fromRPC(rawCell.output.type),
                outPoint:OutPoint.fromRPC(rawCell.out_point),
                data:rawCell.output_data
            };
            this.cells.push(cell);
        }
        return this.cells;
    }

    async getBalance(address) {
        const cells = await this.getCells(address);
        if (!cells.length) return Amount.ZERO;
        return cells
            .map((c) => c.capacity)
            .reduce((sum, cap) => (sum = sum.add(cap)));
    }

    async collect(address, { withData }) {
        const cells = await this.getCells(address);

        if (withData) {
            return cells.filter((c) => !c.isEmpty() && !c.type);
        }

        return cells.filter((c) => c.isEmpty() && !c.type);
    }
}
module.exports={
    SDCollector
}