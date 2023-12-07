echo REM When mining to a local node, you can drop the -s option. > ${1}/mine.bat
echo echo ============================================================ >> ${1}/mine.bat
echo echo = Running Karlsen Miner with default .bat. Edit to configure = >> ${1}/mine.bat
echo echo ============================================================ >> ${1}/mine.bat
echo :start >> ${1}/mine.bat
echo ${1}.exe -a karlsen:qq6xkkf2t209dmlhmffshdagnkkhzufc5xdt03c3psgxv5yk7panqzwdm0pwp -s n.mainnet-dnsseed-1.karlsencoin.com >> ${1}/mine.bat
echo goto start >> ${1}/mine.bat