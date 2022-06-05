# Redis Management System for Live Tickers

## Key Representation
**Root key for an asset:**  
<ScripName>:<Exchange>:<?Expiry>:<?"FUTURE"|Strike>:<?"CE"|"PE">  
? -> Only if applicable

`ScripName` and presence of other values such as expiry `Expiry`  
indicate if the values are spot of options.  
A single `Exchange` place-holder should satisfacorily indicate the  
asset-class.

**Overall Key:**  
<Root Key>:<?STATS|SCRATCH|CANDLES>:<?TIMESTAMP>
Suffix to the root key determines the purpose:
- STATS: For statistics and metrics to determine the healthof the ticker.
- CANDLES: with timestamp, a candle containing OHLCV values.
- SCRATCH: Scratch pad for any operations related to the scrip that are  
	required. To avoid disturbing the schema of other sub-keys.


# TODO:
- [ ] Live Info from dynamodb
