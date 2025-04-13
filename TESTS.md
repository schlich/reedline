```mermaid
graph TD
    %% Level 2: Integration Tests (Engine + Helix + Buffer)
    subgraph "L2" ["Integration (Engine + Helix + Buffer)"]
        direction TB

        subgraph I-Basic ["Basic Interaction"]
            I1["I1: Init Engine w/ Helix"] --> I2["I2: Mode Switching Flow OK"]
        end

        subgraph I-Insert ["Insert Workflow"]
            I3["I3: Type Chars OK"] --> I4["I4: Paste OK"]
        end

        subgraph I-Normal ["Normal Workflow"]
            I5["I5: Basic Move OK"] --> I6["I6: Word/Line Move OK"]
            I6 --> I7["I7: Delete Char (x) OK"]
            I7 --> I8["I8: Delete Motion (dw) OK"]
            I8 --> I9["I9: Yank/Paste (yw, p) OK"]
        end

        subgraph I-Select ["Select Workflow"]
            I10["I10: Enter/Exit Select OK"] --> I11["I11: Select Move Adjusts Range OK"]
            I11 --> I12["I12: Select Delete/Yank OK"]
        end

        subgraph I-Complex ["Complex Commands"]
            I13["I13: Counts Work (3l, 2dw) OK"]
        end

        %% High-level Integration Flow
        I1 --> I2
        I2 --> I3
        I2 --> I5
        I2 --> I10
        I5 --> I13
        %% Counts depend on basic actions
    end

    %% Level 1: Unit Tests (Helix Adapter Logic)
    subgraph "L1" ["L1: Unit (Helix Adapter: Modalkit Action -> ReedlineEvent)"]

        direction TB

        subgraph U-Modes ["Mode Management"]
            U1["U1: Initial State Normal"] --> U2["U2: edit_mode() Reports"]
            U2 --> U3[Normal->Insert ('i')"] --> U4[Insert->Normal ('Esc')"]
            U4 --> U5["U5: Normal->Select ('v')"] --> U6[Select->Normal ('Esc')"]
        end

        subgraph U-Insert ["Insert Actions"]
            U7["U7: InsertChar Translate"] --> U8["U8: Paste Translate"]
        end

        subgraph U-NormalMove ["Normal Movement"]
            U9["U9: Basic Move Translate (h,j,k,l)"] --> U10["U10: Word Move Translate (w,b,e)"]
            U10 --> U11["U11: Line Move Translate (0,^,$)"] --> U12["U12: Char Search Translate (f,t)"]
        end

        subgraph U-NormalDelete ["Normal Deletion"]
            U13["U13: Delete Char Translate (x)"] --> U14["U14: Delete Motion Translate (dw)"]
            U14 --> U15["U15: Delete Line Translate (dd)"]
        end

        subgraph U-NormalYankPaste ["Normal Yank/Paste"]
            U16["U16: Yank Motion Translate (yw)"] --> U17["U17: Paste Translate (p)"]
        end

        subgraph U-SelectMove ["Select Movement"]
            U18["U18: Select Move Translate"]
        end

        subgraph U-SelectAction ["Select Action"]
            U19["U19: Select Delete Translate"] --> U20["U20: Select Yank Translate"]
        end

        subgraph U-Counts ["Counts"]
            U21["U21: Count Translate"]
        end
    end

    %% Level 0: Unit Tests (Modalkit Config Verification)
    subgraph "L0" ["L0: Unit (Modalkit Config Verification)"]
        direction TB

        C1["C1: Mode Defs Exist"]
        C2["C2: Normal ModeChange Maps OK"]
        C3["C3: Normal Action Maps OK"]
        C4["C4: Insert Action Maps OK"]
        C5["C5: Insert ModeChange Maps OK"]
        C6["C6: Select Action Maps OK"]
        C7["C7: Select Motion Maps OK"]

        C1 --> C2; C1 --> C3; C1 --> C4; C1 --> C5; C1 --> C6; C1 --> C7;
    end

    %% Dependencies Between Levels (Illustrative)
    I2 --> U-Modes; I3 --> U7; I4 --> U8;
    I5 --> U9; I6 --> U10; I6 --> U11;
    I7 --> U13; I8 --> U14; I9 --> U16; I9 --> U17;
    I10 --> U-Modes; I11 --> U18; I12 --> U19; I12 --> U20;
    I13 --> U21;

    U-Modes --> C1; U-Modes --> C2; U-Modes --> C5;
    U7 --> C4; U8 --> C4; 
    %% Insert actions need Insert maps
    U9 --> C3; U10 --> C3; U11 --> C3; U12 --> C3; 
    %% Normal moves need Normal maps
    U13 --> C3; U14 --> C3; U15 --> C3; 
    %% Normal deletes need Normal maps
    U16 --> C3; U17 --> C3; 
    %% Normal Yank/Paste need Normal maps
    U18 --> C7; 
    %% Select moves need Select motion maps
    U19 --> C6; U20 --> C6; 
    %% Select actions need Select action maps
    U21 --> C3; 
    %% Counts likely apply to Normal actions
  ```