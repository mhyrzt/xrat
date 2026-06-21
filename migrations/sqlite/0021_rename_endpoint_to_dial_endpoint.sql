ALTER TABLE connection_tests
RENAME COLUMN endpoint_ip TO dial_endpoint_ip;
ALTER TABLE connection_tests
RENAME COLUMN endpoint_location TO dial_endpoint_location;
ALTER TABLE connection_tests
RENAME COLUMN endpoint_country TO dial_endpoint_country;
ALTER TABLE connection_tests
RENAME COLUMN endpoint_asn TO dial_endpoint_asn;
ALTER TABLE connection_tests ADD COLUMN dial_endpoint_geoip_source TEXT;
ALTER TABLE connection_tests ADD COLUMN dial_endpoint_fronting TEXT;
