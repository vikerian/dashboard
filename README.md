# dashboard
Learning project - Basic web dashboard using some more types of DB

# This project is learning project

- project generates dashboard, which is connected to multiple data sources
- data sources:
    - postgresql databse with timescaledb extension
    - valkey database
    - mosquitto mqtt server
    - manticore search which is automaticaly indexing data from postgresql

- for now - working with inserted mock data

## TODO: 
    - create configurable database layer, for now we have only few things in db. 
    - Our app should be use any sql scheme we provide in configuration.
