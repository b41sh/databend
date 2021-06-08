SELECT sum(number) from numbers_mt(10000);
SELECT min(number) from numbers_mt(10000);
SELECT max(number) from numbers_mt(10000);
SELECT avg(number) from numbers_mt(10000);
SELECT count(number) from numbers_mt(10000);
SELECT sum(number)/count(number) from numbers_mt(10000);
SELECT argMin(number, number) from numbers_mt(10000);
SELECT argMin(a, b) from (select number + 5 as a, number - 5 as b from numbers_mt(10000));
SELECT argMin(b, a) from (select number + 5 as a, number - 5 as b from numbers_mt(10000));
SELECT argMax(number, number) from numbers_mt(10000);
SELECT argMax(a, b) from (select number + 5 as a, number - 5 as b from numbers_mt(10000));
SELECT argMax(b, a) from (select number + 5 as a, number - 5 as b from numbers_mt(10000));

-- test argMax, argMin fro String
 SELECT argMax(a, b) from (select number + 5 as a, cast(number as varchar(255)) as b from numbers_mt(10000)) ;
 SELECT argMax(b, a) from (select number + 5 as a, cast(number as varchar(255)) as b from numbers_mt(10000)) ;


 select count(distinct number, number + 1 , number + 3 ) from ( select number % 100 as number from numbers(100000));
 select count(distinct 3) from numbers(10000);
 select uniq(number, number + 1 , number + 3 )  =  count(distinct number, number + 1 , number + 3 ) from ( select number % 100 as number from numbers(100000));