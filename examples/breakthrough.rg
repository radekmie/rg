// Breakthrough (concept in RG 0.0.8)

domain players = {white,black};
domain scores = {0,100};
domain pieces = {e,b,w};
domain squares = {v00,v10,v20,v30,v40,v50,v60,v70,
                  v01,v11,v21,v31,v41,v51,v61,v71,
                  v02,v12,v22,v32,v42,v52,v62,v72,
                  v03,v13,v23,v33,v43,v53,v63,v73,
                  v04,v14,v24,v34,v44,v54,v64,v74,
                  v05,v15,v25,v35,v45,v55,v65,v75,
                  v06,v16,v26,v36,v46,v56,v66,v76,
                  v07,v17,v27,v37,v47,v57,v67,v77};
domain squares_with_null = {null,
             v00,v10,v20,v30,v40,v50,v60,v70,
             v01,v11,v21,v31,v41,v51,v61,v71,
             v02,v12,v22,v32,v42,v52,v62,v72,
             v03,v13,v23,v33,v43,v53,v63,v73,
             v04,v14,v24,v34,v44,v54,v64,v74,
             v05,v15,v25,v35,v45,v55,v65,v75,
             v06,v16,v26,v36,v46,v56,v66,v76,
             v07,v17,v27,v37,v47,v57,v67,v77};

constant left: squares -> squares_with_null,
               default = null,
               left(v10)=v00,left(v20)=v10,left(v30)=v20,left(v40)=v30,left(v50)=v40,left(v60)=v50,left(v70)=v60,
               left(v11)=v01,left(v21)=v11,left(v31)=v21,left(v41)=v31,left(v51)=v41,left(v61)=v51,left(v71)=v61,
               left(v12)=v02,left(v22)=v12,left(v32)=v22,left(v42)=v32,left(v52)=v42,left(v62)=v52,left(v72)=v62,
               left(v13)=v03,left(v23)=v13,left(v33)=v23,left(v43)=v33,left(v53)=v43,left(v63)=v53,left(v73)=v63,
               left(v14)=v04,left(v24)=v14,left(v34)=v24,left(v44)=v34,left(v54)=v44,left(v64)=v54,left(v74)=v64,
               left(v15)=v05,left(v25)=v15,left(v35)=v25,left(v45)=v35,left(v55)=v45,left(v65)=v55,left(v75)=v65,
               left(v16)=v06,left(v26)=v16,left(v36)=v26,left(v46)=v36,left(v56)=v46,left(v66)=v56,left(v76)=v66,
               left(v17)=v07,left(v27)=v17,left(v37)=v27,left(v47)=v37,left(v57)=v47,left(v67)=v57,left(v77)=v67;
constant right: squares -> squares_with_null,
                default = null,
                right(v00)=v10,right(v10)=v20,right(v20)=v30,right(v30)=v40,right(v40)=v50,right(v50)=v60,right(v60)=v70,
                right(v01)=v11,right(v11)=v21,right(v21)=v31,right(v31)=v41,right(v41)=v51,right(v51)=v61,right(v61)=v71,
                right(v02)=v12,right(v12)=v22,right(v22)=v32,right(v32)=v42,right(v42)=v52,right(v52)=v62,right(v62)=v72,
                right(v03)=v13,right(v13)=v23,right(v23)=v33,right(v33)=v43,right(v43)=v53,right(v53)=v63,right(v63)=v73,
                right(v04)=v14,right(v14)=v24,right(v24)=v34,right(v34)=v44,right(v44)=v54,right(v54)=v64,right(v64)=v74,
                right(v05)=v15,right(v15)=v25,right(v25)=v35,right(v35)=v45,right(v45)=v55,right(v55)=v65,right(v65)=v75,
                right(v06)=v16,right(v16)=v26,right(v26)=v36,right(v36)=v46,right(v46)=v56,right(v56)=v66,right(v66)=v76,
                right(v07)=v17,right(v17)=v27,right(v27)=v37,right(v37)=v47,right(v47)=v57,right(v57)=v67,right(v67)=v77;
constant up: squares -> squares_with_null,
             default = null,
             up(v01)=v00,up(v02)=v01,up(v03)=v02,up(v04)=v03,up(v05)=v04,up(v06)=v05,up(v07)=v06,
             up(v11)=v10,up(v12)=v11,up(v13)=v12,up(v14)=v13,up(v15)=v14,up(v16)=v15,up(v17)=v16,
             up(v21)=v20,up(v22)=v21,up(v23)=v22,up(v24)=v23,up(v25)=v24,up(v26)=v25,up(v27)=v26,
             up(v31)=v30,up(v32)=v31,up(v33)=v32,up(v34)=v33,up(v35)=v34,up(v36)=v35,up(v37)=v36,
             up(v41)=v40,up(v42)=v41,up(v43)=v42,up(v44)=v43,up(v45)=v44,up(v46)=v45,up(v47)=v46,
             up(v51)=v50,up(v52)=v51,up(v53)=v52,up(v54)=v53,up(v55)=v54,up(v56)=v55,up(v57)=v56,
             up(v61)=v60,up(v62)=v61,up(v63)=v62,up(v64)=v63,up(v65)=v64,up(v66)=v65,up(v67)=v66,
             up(v71)=v70,up(v72)=v71,up(v73)=v72,up(v74)=v73,up(v75)=v74,up(v76)=v75,up(v77)=v76;
constant down: squares -> squares_with_null,
               default = null,
               down(v00)=v01,down(v01)=v02,down(v02)=v03,down(v03)=v04,down(v04)=v05,down(v05)=v06,down(v06)=v07,
               down(v10)=v11,down(v11)=v12,down(v12)=v13,down(v13)=v14,down(v14)=v15,down(v15)=v16,down(v16)=v17,
               down(v20)=v21,down(v21)=v22,down(v22)=v23,down(v23)=v24,down(v24)=v25,down(v25)=v26,down(v26)=v27,
               down(v30)=v31,down(v31)=v32,down(v32)=v33,down(v33)=v34,down(v34)=v35,down(v35)=v36,down(v36)=v37,
               down(v40)=v41,down(v41)=v42,down(v42)=v43,down(v43)=v44,down(v44)=v45,down(v45)=v46,down(v46)=v47,
               down(v50)=v51,down(v51)=v52,down(v52)=v53,down(v53)=v54,down(v54)=v55,down(v55)=v56,down(v56)=v57,
               down(v60)=v61,down(v61)=v62,down(v62)=v63,down(v63)=v64,down(v64)=v65,down(v65)=v66,down(v66)=v67,
               down(v70)=v71,down(v71)=v72,down(v72)=v73,down(v73)=v74,down(v74)=v75,down(v75)=v76,down(v76)=v77;

constant white_or_empty: pieces -> {F,T}, default = F, white_or_empty(w)=T, white_or_empty(e)=T;
constant black_or_empty: pieces -> {F,T}, default = F, black_or_empty(b)=T, black_or_empty(e)=T;

var white: scores, visible={white,black};
var black: scores, visible={white,black};
var board: squares -> pieces, default = e, visible={white,black};
var pos: squares, visible={white,black};

// Init in state 0
0,1: [board = {v00=b,v10=b,v20=b,v30=b,v40=b,v50=b,v60=b,v70=b,
               v01=b,v11=b,v21=b,v31=b,v41=b,v51=b,v61=b,v71=b,
               v06=w,v16=w,v26=w,v36=w,v46=w,v56=w,v66=w,v76=w,
               v07=w,v17=w,v27=w,v37=w,v47=w,v57=w,v67=w,v77=w}];
1,10: ->white;

10,11: [pos=*];
11,12: {? board[pos]==w};
12,13: [board[pos]=e];
13,14: {? up(pos)!=null};
14,15: [pos=up(pos)];
// Branch left
15,16: {? left(pos)!=null};
16,18: [pos=left(pos)];
// Branch right
15,17: {? right(pos)!=null};
17,18: [pos=right(pos)];
// Branch up
15,18: {? board[pos]==e};
// Join
18,19: {? black_or_empty(board[pos])==T};
19,20: [board[pos]=w];
20,21: ->>; // Keeper switch

21,22: {? up(pos)==null}; // Reach line win
21,23: {> 40 !-> 51}; // Opponent has no legal move
22,24: ; // Empty transition
23,24: ;
24,25: [white=100];
25,26: [black=0];
26,27: ->>; // Keeper ending with no move
21,28: {? up(pos)!=null};
28,29: {> 40 ?-> 51};
29,40: ->black;

40,41: [pos=*];
41,42: {? board[pos]==b};
42,43: [board[pos]=e];
43,44: {? down(pos)!=null};
44,45: [pos=down(pos)];
// Branch left
45,46: {? left(pos)!=null};
46,48: [pos=left(pos)];
// Branch right
45,47: {? right(pos)!=null};
47,48: [pos=right(pos)];
// Branch up
45,48: {? board[pos]==e};
// Join
48,49: {? white_or_empty(board[pos])==T};
49,50: [board[pos]=b];
50,51: ->>; // Keeper switch

51,52: {? down(pos)==null}; // Reach line win
51,53: {> 10 !-> 21}; // Opponent has no legal move
52,54: ;
53,54: ;
54,55: [white=0];
55,56: [black=100];
56,57: ->>; // Keeper ending with no move
51,58: {? down(pos)!=null};
58,59: {> 10 ?-> 21};
59,10: ->white;
