//*****************************************************************
// Title: Random Access Machine Emulator
// Author: Osami Yamamoto (osami@ccmfs.meijo-u.ac.jp)
// Id: $Id: RAM.java,v 208bcd7fe429 2023/04/25 09:58:01 osami $
//*****************************************************************
import java.io.FileReader;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.FileNotFoundException;

public class RAM{
    final static int LOAD = 1;
    final static int STORE = 2;
    final static int ADD = 3;
    final static int SUB = 4;
    final static int MULT = 5;
    final static int DIV = 6;
    final static int READ = 7;
    final static int WRITE = 8;
    final static int JUMP = 9;
    final static int JGTZ = 10;
    final static int JZERO = 11;
    final static int HALT = 12;
    final static int SJ = 13;
    final static int max_prog_len = 4000;
    final static int max_number_of_labels = 200;
    final static int op_offset = 67108864;
    final static int max_memory = 4096;
    final static int max_input = 4096;
    static int [] memory = new int [max_memory];
    static int [] program = new int [max_prog_len];
    static String [] labels = new String[max_number_of_labels];
    static int [] label_num = new int [max_number_of_labels];
    static int [] input_tape = new int [max_input];
    static int n_label = 0;
    static int number_of_lines = 0;
    static int n_input = 0;
    static int p_tape = 0;

    public static void print_banner(){
		String [] banner = {
			"================================================",
			"| Random Access Machine Emulator",
			"| Osami Yamamoto (osami@ccmfs.meijo-u.ac.jp)",
			"| $Date: 2023/04/25 09:58:01 $, $Revision: 208bcd7fe429 $",
			"================================================" 
		};
		for (int i = 0; i < banner.length; i++ ){
			System.out.println( banner[ i ] );
		} /* for */
    }

    public static void print_prog( String fname )
	throws FileNotFoundException{
		FileReader fp  = new FileReader( fname );
		BufferedReader br = new BufferedReader( fp );
		try{
			int no = 0;
			while ( true ){
			String line = br.readLine();
			if (line == null) break;
			String s = "";
			if (no < 10 ) s += " ";
			if (no < 100 ) s += " ";
			if (no < 1000 ) s += " ";
			System.out.println( s + no + ":" + line );
			no += 1;
			}
			fp.close();
		} catch (IOException e){
		}
    }
    public static void compile( String fname )
	throws FileNotFoundException{
	FileReader fp  = new FileReader( fname );
	BufferedReader br = new BufferedReader( fp );
	try{
	    int line_number = 0;
	    String linex;
	    while ( true ){
		String line = br.readLine();
		if (line == null) break;
		line = line.trim();
		if (line.indexOf( ';' ) >= 0)
		    line = line.substring(0, line.indexOf( ';' ));
		if (line.length() == 0 ) continue;
		if (line.indexOf( ':' ) >= 0 ){
		    linex = line.substring(line.indexOf( ':') + 1).trim();
		    line = line.substring(0, line.indexOf( ':'));
		    labels[ n_label ] = line;
		    label_num[ n_label ] = line_number;
		    n_label += 1;
		    if (linex.length() == 0)
			line_number -= 1;
		}
		line_number += 1;
	    } /* while */
// 	    for (int i = 0; i < n_label; i++ ){
// 		System.out.println( i + ":" +
// 				    labels[ i ] + ": " + label_num[ i ] );
// 	    }
	    fp.close();
	} catch (IOException e){
	}
	fp  = new FileReader( fname );
	br = new BufferedReader( fp );
	try{
	    int line_number = 0;
	    int n_label = 0;
	    String linex;
	    while ( true ){
		String line = br.readLine();
		if (line == null) break;
		line = line.trim();
		if (line.indexOf( ';' ) >= 0)
		    line = line.substring(0, line.indexOf( ';' ));
		if (line.length() == 0 ) continue;
		if (line.indexOf( ':' ) >= 0 ){
		    line = line.substring(line.indexOf( ':') + 1).trim();
		    if (line.length() == 0 ) continue;
		}
		program[ line_number ] = encode(line);
		// System.out.println( line_number + ": " + line );
		line_number += 1;
	    } /* while */
	    fp.close();
	    number_of_lines = line_number;
	} catch (IOException e){
	}
// 	for (int i = 0; i < number_of_lines; i++ ){
// 	    int nn = program[ i ];
// 	    System.out.println( i + ":" + (nn / op_offset) +
// 				":" + (nn % op_offset) );
// 	}  /* for */
    }
    public static int encode( String line ){
	if (line.startsWith( "LOAD" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return LOAD * op_offset + opx( operand );
	} else if (line.startsWith( "STORE" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return STORE * op_offset + opx( operand );
	} else if (line.startsWith( "ADD" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return ADD * op_offset + opx( operand );
	} else if (line.startsWith( "SUB" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return SUB * op_offset + opx( operand );
	} else if (line.startsWith( "MULT" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return MULT * op_offset + opx( operand );
	} else if (line.startsWith( "DIV" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return DIV * op_offset + opx( operand );
	} else if (line.startsWith( "READ" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return READ * op_offset + opx( operand );
	} else if (line.startsWith( "WRITE" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    return WRITE * op_offset + opx( operand );
	} else if (line.startsWith( "JUMP" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    for (int i = 0; i < n_label; i++ ){
		if (labels[i].equals(operand))
		    return JUMP * op_offset + label_num[ i ];
	    }
	    System.out.println( "Unknown label: " + operand );
	    System.exit( 1 );
	} else if (line.startsWith( "JGTZ" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    for (int i = 0; i < n_label; i++ ){
		if (labels[i].equals(operand))
		    return JGTZ * op_offset + label_num[ i ];
	    }
	    System.out.println( "Unknown label: " + operand );
	    System.exit( 1 );
	} else if (line.startsWith( "JZERO" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    for (int i = 0; i < n_label; i++ ){
		if (labels[i].equals(operand))
		    return JZERO * op_offset + label_num[ i ];
	    }
	    System.out.println( "Unknown label: " + operand );
	    System.exit( 1 );
	} else if (line.startsWith( "HALT" )){
	    return HALT * op_offset;
	} else if (line.startsWith( "SJ" )){
	    String operand = line.substring( line.indexOf(' ') ).trim();
	    String operand1 = operand.substring(0, operand.indexOf(','));
	    String rest = operand.substring(operand.indexOf(',') + 1);
	    String operand2 = rest.substring(0, rest.indexOf(','));
	    String operand3 = rest.substring(rest.indexOf(',') + 1);
	    operand1 = operand1.trim();
	    operand2 = operand2.trim();
	    operand3 = operand3.trim();
	    int label_nn = 0;
	    for (int i = 0; i < n_label; i++ ){
		if (labels[i].equals(operand3)){
		    label_nn = label_num[ i ];
		    break;
		} /* if */
	    } /* for */
// 	    System.out.println( "operand1 = " + operand1 );
// 	    System.out.println( "operand2 = " + operand2 );
// 	    System.out.println( "operand3 = " + operand3 );
// 	    System.out.println( "label_nn = " + label_nn );
	    return SJ * op_offset +
		(opx( operand1) * 256 + opx( operand2 )) * 256 + label_nn;
	} else {
	    System.out.println( "Unknown Command: " + line );
	    System.exit( 1 );
	}
	return 0;
    }
    static int opx(String operand){
	int x = 0;
	int sign = 0;
	int dir = 0;
	if (operand.charAt( 0 ) == '=' ){
	    int y = Integer.parseInt(operand.substring( 1 ));
	    if (y < 0 ){
		x = -y;
		sign = 1;
	    } else {
		x = y;
	    }
	} else if (operand.charAt( 0 ) == '*' ){
	    int y = Integer.parseInt(operand.substring( 1 ));
	    dir = 1;
	    x = y;
	} else {
	    int y = Integer.parseInt(operand);
	    x = y;
	    dir = 2;
	}
	return (x * 2 + sign) * 4 + dir;
    }
    static void interp(){
	int pc = 0;
	int count = 0;
	while (true ){
	    /* System.out.println( "pc = " + pc ); */
	    int op = program[ pc ];
	    int command = op / op_offset;
	    count += 1;
	    if ( command == LOAD ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    memory[ 0 ] = operand;
		else if (dir == 1)
		    memory[ 0 ] = memory[ memory[ operand]];
		else 
		    memory[ 0 ] = memory[ operand ];
	    } else if (command == STORE ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    operand = memory[ 0 ];
		else if (dir == 1)
		    memory[ memory[ operand]] = memory[ 0 ];
		else 
		    memory[ operand ] = memory[ 0 ];
	    } else if ( command == ADD ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    memory[ 0 ] += operand;
		else if (dir == 1)
		    memory[ 0 ] += memory[ memory[ operand]];
		else 
		    memory[ 0 ] += memory[ operand ];
	    } else if ( command == SUB ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    memory[ 0 ] -= operand;
		else if (dir == 1)
		    memory[ 0 ] -= memory[ memory[ operand]];
		else 
		    memory[ 0 ] -= memory[ operand ];
	    } else if ( command == MULT ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    memory[ 0 ] *= operand;
		else if (dir == 1)
		    memory[ 0 ] *= memory[ memory[ operand]];
		else 
		    memory[ 0 ] *= memory[ operand ];
	    } else if ( command == DIV ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    memory[ 0 ] /= operand;
		else if (dir == 1)
		    memory[ 0 ] /= memory[ memory[ operand]];
		else 
		    memory[ 0 ] /= memory[ operand ];
	    } else if (command == JUMP ){
		int operand = (op % op_offset);
		pc = operand;
		continue;
	    } else if (command == JGTZ ){
		int operand = (op % op_offset);
		if (memory[0] > 0){
		    pc = operand;
		    continue;
		} /* if */
	    } else if (command == JZERO ){
		int operand = (op % op_offset);
		if (memory[0] == 0){
		    pc = operand;
		    continue;
		} /* if */
	    } else if (command == WRITE ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		if (dir == 0)
		    System.out.println( "WRITE: " + operand);
		else if (dir == 1)
		    System.out.println( "WRITE: " +
					memory[ memory[ operand]] );
		else 
		    System.out.println( "WRITE: " +
					memory[ operand ]);
	    } else if (command == READ ){
		int operand = op % op_offset;
		int dir = operand % 4;
		operand = operand / 4;
		int sign = operand % 2;
		operand = operand / 2;
		if (sign == 1) operand = -operand;
		int val = 0;
		if (val < n_input ){
		    val = input_tape[ p_tape ];
		    p_tape += 1;
		}
		if (dir == 0){
		    String msg = "Error: You cannot read from a number.";
		    System.out.println(msg);
		    System.exit( 1 );
		}
		else if (dir == 1)
		    memory[ memory[ operand]] = val;
		else 
		    memory[ operand ] = val;
	    } else if (command == SJ ){
		int operand = op % op_offset;
		int operand3 = operand % 256;
		int operand2 = (operand / 256) % 256;
		int operand1 = (operand / 256) / 256;
		int val2 = 0;
		int dir1 = operand1 % 4;
		operand1 = operand1 / 4;
		int sign1 = operand1 % 2;
		operand1 = operand1 / 2;
		if (sign1 == 1) operand1 = -operand1;
		int dir2 = operand2 % 4;
		operand2 = operand2 / 4;
		int sign2 = operand2 % 2;
		operand2 = operand2 / 2;
		if (sign2 == 1) operand1 = -operand1;
		if (dir2 == 0)
		    val2 = operand2;
		else if (dir2 == 1)
		    val2 = memory[ memory[ operand2]];
		else 
		    val2 = memory[ operand2 ];
		if (dir1 == 0)
		    System.out.println( "Error!" );
		else if (dir1 == 1)
		    memory[ memory[ operand1]] -= val2;
		else 
		    memory[ operand1 ] -= val2;
		if (dir1 == 1){
		    if (memory[ memory[ operand1]] == 0){
			pc = operand3;
			continue;
		    }
		} else if (dir1 > 1){
		    if (memory[ operand1 ] == 0){
			pc = operand3;
			continue;
		    }
		}
	    } else if (command == HALT )
		break;
	    pc += 1;
	}
	System.out.println( "number of steps = " + count );
    }
    public static void init_read_file() throws FileNotFoundException{
	final String fname = "input_file";
	FileReader fp = new FileReader( fname );
	BufferedReader br = new BufferedReader( fp );
	try{
	    int i = 0;
	    while (true){
		String line = br.readLine();
		if (line == null) break;
		line = line.trim();
		int n = Integer.parseInt( line );
		input_tape[ i ] = n;
		i += 1;
	    } /* while */
	    n_input = i;
	    fp.close();
	} catch (IOException e ){}
    }
    public static void main( String [] args )
	throws FileNotFoundException{
	String fname = args[0];
	print_banner();
	try {
	    init_read_file();
	    print_prog( fname );
	    compile( fname );
	    interp();
	} catch (FileNotFoundException e ){
	    System.out.println( "Error: " + e);
	} /* try */
    } /* main */
} /* class RAM */
