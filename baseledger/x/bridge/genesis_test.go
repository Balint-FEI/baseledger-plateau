package bridge_test

// import (
// 	"testing"

// 	keepertest "github.com/Baseledger/baseledger/testutil/keeper"
// 	"github.com/Baseledger/baseledger/testutil/nullify"
// 	bridge "github.com/Baseledger/baseledger/x/bridge"
// 	"github.com/Baseledger/baseledger/x/bridge/types"
// 	"github.com/stretchr/testify/require"
// )

// func TestGenesis(t *testing.T) {
// 	genesisState := types.GenesisState{
// 		Params: types.DefaultParams(),

// 		OrchestratorValidatorAddressList: []types.OrchestratorValidatorAddress{
// 		{
// 			OrchestratorAddress: "0",
// },
// 		{
// 			OrchestratorAddress: "1",
// },
// 	},
// 	// this line is used by starport scaffolding # genesis/test/state
// 	}

// 	k, ctx := keepertest.BaseledgerbridgeKeeper(t)
// 	bridge.InitGenesis(ctx, *k, genesisState)
// 	got := bridge.ExportGenesis(ctx, *k)
// 	require.NotNil(t, got)

// 	nullify.Fill(&genesisState)
// 	nullify.Fill(got)

// 	require.ElementsMatch(t, genesisState.OrchestratorValidatorAddressList, got.OrchestratorValidatorAddressList)
// // this line is used by starport scaffolding # genesis/test/assert
// }
