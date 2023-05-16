app: 
    @-mkdir applibs
    @for file in `ls ./apps`; do \
    rustc --edition 2021 --out-dir ./applibs ./apps/$file; \
    done
#rustc --edition 2021 --out-dir ./applibs ./apps/wc.rs

clean_app:
    @rm -rf applibs 

clean: clean_app