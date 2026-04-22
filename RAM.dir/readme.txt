
RAMエミュレータの使い方
======================

授業で説明したRAMの命令によって書かれたプログラムを解釈実行する
プログラムです．100%JAVAによって書かれています．利用法は簡単で，
RAMのプログラムをファイルの書き，そのファイルを指定して以下のように
実行します．

osami-2:yama571> java RAM test1.ram
================================================
| Random Access Machine Emulator
| Osami Yamamoto (osami@ccmfs.meijo-u.ac.jp)
| $Date: 2023/04/25 09:58:01 $, $Revision: 208bcd7fe429 $
================================================
   0:
   1:start:
   2:	LOAD =1
   3:	STORE 3
   4:	LOAD =10
   5:loop:
   6:	SUB 3
   7:	WRITE 0
   8:	JGTZ loop
   9:	HALT
WRITE: 9
WRITE: 8
WRITE: 7
WRITE: 6
WRITE: 5
WRITE: 4
WRITE: 3
WRITE: 2
WRITE: 1
WRITE: 0
osami-2:yama572> 

授業で説明したSJ命令も実行させることができます．プログラム中の漢字
コードは場合によっては誤動作の原因となります．JISコードに変換してから
実行させれば問題は起きないようです．Windows上でも問題なく動くはずです．

山本修身
Fri Apr 16 12:36:09 JST 2010



