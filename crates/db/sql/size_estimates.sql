SELECT relname, reltuples AS estimate FROM pg_class 
where (
    reltuples >= 0 and  
    relname NOT LIKE 'pg_%' and 
    relname NOT LIKE 'sql\_%'  
    and relname NOT LIKE '%\_pkey' 
    and relname NOT LIKE '%\_id_seq' 
    and relname NOT LIKE '%\_index'
)