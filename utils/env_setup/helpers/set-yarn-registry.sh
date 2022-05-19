
cd $PROGRAM_ROOT

if [ "$1" = "reset" ]
then
  yarn config set npmRegistryServer https://$NPM_LIVE_REGISTRY
  yarn config set unsafeHttpWhitelist --json '[]'
else
  yarn config set npmRegistryServer http://$NPM_LOCAL_REGISTRY
  yarn config set unsafeHttpWhitelist --json '["${NPM_LOCAL_REGISTRY}"]'
fi
